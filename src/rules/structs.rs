//! Obsess over your data structures. Put behavior on the type it belongs to.
//! <https://almaju.github.io/blog/docs/fundamentals/modeling/structs>
//! <https://almaju.github.io/blog/docs/fundamentals/modeling/method-ownership>

use std::collections::BTreeMap;

use syn::spanned::Spanned;
use syn::visit::Visit;

use crate::rule::Rule;
use crate::rules::{Check, Context, Findings, has_cfg_test, has_test_attr, type_ident, unwrapped};

pub struct Structs;

impl Check for Structs {
    fn run(&self, cx: &Context) -> Findings {
        let mut visitor = Visitor {
            cx,
            findings: Findings::default(),
            in_trait_impl: false,
            methods_per_type: BTreeMap::new(),
            test_depth: usize::from(cx.in_test_file()),
        };
        visitor.visit_file(&cx.file.ast);
        visitor.report_oversized_impls();
        visitor.findings
    }
}

struct Visitor<'a> {
    cx: &'a Context<'a>,
    findings: Findings,
    /// Inside `impl Trait for T`, where the signatures are the trait's choice.
    in_trait_impl: bool,
    /// Inherent methods per self type, with the span of the first impl block.
    methods_per_type: BTreeMap<String, (proc_macro2::Span, usize)>,
    test_depth: usize,
}

impl Visitor<'_> {
    /// A free function whose primary parameter or return type is one of your
    /// own types has a home. It is not here.
    fn check_free_function(&mut self, node: &syn::ItemFn) {
        if self.test_depth > 0 || node.sig.ident == "main" || node.sig.abi.is_some() {
            return;
        }
        if node.attrs.iter().any(|attr| attr.path().is_ident("no_mangle")) {
            return;
        }
        let generics: Vec<String> = node
            .sig
            .generics
            .type_params()
            .map(|param| param.ident.to_string())
            .collect();
        let owner = |ty: &syn::Type| -> Option<String> {
            let ident = type_ident(unwrapped(ty))?.to_string();
            (self.cx.local_types.contains(&ident) && !generics.contains(&ident)).then_some(ident)
        };
        let primary = node.sig.inputs.iter().find_map(|input| match input {
            syn::FnArg::Typed(arg) => {
                owner(&arg.ty).map(|name| (name, self.cx.file.text_of(&*arg.pat).to_string()))
            }
            syn::FnArg::Receiver(_) => None,
        });
        let name = node.sig.ident.to_string();
        let returned = match &node.sig.output {
            syn::ReturnType::Type(_, output) => owner(output),
            syn::ReturnType::Default => None,
        };
        let (message, help) = match (primary, returned) {
            (Some((ty, param)), Some(out)) if out != ty => (
                format!(
                    "`{name}` takes `{param}: {ty}` and returns `{out}`: it belongs on one of them, not in between"
                ),
                format!("make it `{param}.{name}(..)` in `impl {ty}`, or a constructor in `impl {out}`"),
            ),
            (Some((ty, param)), _) => (
                format!(
                    "`{name}` takes `{param}: {ty}`: it is a method on `{ty}` that has not been written yet"
                ),
                format!("move it into `impl {ty}` and call `{param}.{name}(..)`"),
            ),
            (None, Some(ty)) => (
                format!("`{name}` returns `{ty}`: constructors belong on the type they construct"),
                format!("move it into `impl {ty}` as an associated function"),
            ),
            (None, None) => return,
        };
        self.findings.report_with_help(
            self.cx,
            Rule::FreeFunction,
            node.sig.ident.span(),
            message,
            Some(help),
        );
    }

    fn check_parameter_count(&mut self, sig: &syn::Signature) {
        if self.in_trait_impl {
            return;
        }
        let count = sig
            .inputs
            .iter()
            .filter(|input| matches!(input, syn::FnArg::Typed(_)))
            .count();
        let threshold = self.cx.config.thresholds.too_many_parameters;
        if count <= threshold {
            return;
        }
        self.findings.report_with_help(
            self.cx,
            Rule::TooManyParameters,
            sig.ident.span(),
            format!(
                "`{}` takes {count} parameters: a type that has not been split yet",
                sig.ident
            ),
            Some("parameters that travel together are a struct waiting to be named".to_string()),
        );
    }

    fn report_oversized_impls(&mut self) {
        let threshold = self.cx.config.thresholds.oversized_impl;
        let oversized: Vec<(String, proc_macro2::Span, usize)> = self
            .methods_per_type
            .iter()
            .filter(|(_, (_, count))| *count > threshold)
            .map(|(name, (span, count))| (name.clone(), *span, *count))
            .collect();
        for (name, span, count) in oversized {
            self.findings.report_with_help(
                self.cx,
                Rule::OversizedImpl,
                span,
                format!(
                    "`{name}` has {count} methods: usually several types that have not been separated yet"
                ),
                Some(
                    "ask which methods operate on a subset of the fields; that subset is its own struct"
                        .to_string(),
                ),
            );
        }
    }
}

impl<'ast> Visit<'ast> for Visitor<'_> {
    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        self.check_parameter_count(&node.sig);
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        let is_test = has_test_attr(&node.attrs);
        self.test_depth += usize::from(is_test);
        self.check_parameter_count(&node.sig);
        self.check_free_function(node);
        syn::visit::visit_item_fn(self, node);
        self.test_depth -= usize::from(is_test);
    }

    fn visit_item_impl(&mut self, node: &'ast syn::ItemImpl) {
        let was = self.in_trait_impl;
        self.in_trait_impl = node.trait_.is_some();
        if node.trait_.is_none() && self.test_depth == 0 {
            let name = type_ident(&node.self_ty)
                .map(ToString::to_string)
                .unwrap_or_else(|| self.cx.file.text_of(&node.self_ty).to_string());
            let methods = node
                .items
                .iter()
                .filter(|item| matches!(item, syn::ImplItem::Fn(_)))
                .count();
            let entry = self
                .methods_per_type
                .entry(name)
                .or_insert((node.impl_token.span(), 0));
            entry.1 += methods;
        }
        syn::visit::visit_item_impl(self, node);
        self.in_trait_impl = was;
    }

    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        let is_test = has_cfg_test(&node.attrs);
        self.test_depth += usize::from(is_test);
        syn::visit::visit_item_mod(self, node);
        self.test_depth -= usize::from(is_test);
    }

    fn visit_trait_item_fn(&mut self, node: &'ast syn::TraitItemFn) {
        self.check_parameter_count(&node.sig);
    }
}
