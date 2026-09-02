//! Treat errors as data, not exceptions.
//! <https://almaju.github.io/blog/docs/fundamentals/modeling/errors>

use syn::spanned::Spanned;
use syn::visit::Visit;

use crate::rule::Rule;
use crate::rules::{Check, Context, Findings};

pub struct Errors;

impl Check for Errors {
    fn run(&self, cx: &Context) -> Findings {
        let mut visitor = Visitor {
            cx,
            findings: Findings::default(),
            in_trait_impl: false,
            startup_depth: 0,
        };
        visitor.visit_file(&cx.file.ast);
        visitor.findings
    }
}

struct Visitor<'a> {
    cx: &'a Context<'a>,
    findings: Findings,
    /// Inside `impl Trait for T`, where the signatures are the trait's choice.
    in_trait_impl: bool,
    /// Inside `fn main`, where a missing config file may legitimately abort.
    startup_depth: usize,
}

impl Visitor<'_> {
    fn check_signature(&mut self, sig: &syn::Signature) {
        if self.in_trait_impl || self.startup_depth > 0 {
            return;
        }
        let syn::ReturnType::Type(_, ty) = &sig.output else {
            return;
        };
        let text = self.cx.file.text_of(&**ty);
        let erased = if text.contains("dyn") && text.contains("Error") {
            Some("`Box<dyn Error>`")
        } else if text.contains("anyhow") {
            Some("`anyhow`")
        } else if text.contains("eyre") {
            Some("`eyre`")
        } else {
            None
        };
        let Some(erased) = erased else {
            return;
        };
        self.findings.report_with_help(
            self.cx,
            Rule::UntypedError,
            ty.span(),
            format!(
                "`{}` returns {erased}: callers cannot match on what went wrong, so they will guess",
                sig.ident
            ),
            Some("return an enum of the failures a caller can react to differently".to_string()),
        );
    }
}

impl<'ast> Visit<'ast> for Visitor<'_> {
    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        if self.startup_depth == 0 {
            let method = node.method.to_string();
            if matches!(method.as_str(), "unwrap" | "expect" | "unwrap_err" | "expect_err") {
                self.findings.report_with_help(
                    self.cx,
                    Rule::PanicInProduction,
                    node.method.span(),
                    format!("`.{method}()` in production code is a bet that this call never fails"),
                    Some(
                        "propagate with `?` or match on the failure; panics are for startup and programmer errors"
                            .to_string(),
                    ),
                );
            }
        }
        syn::visit::visit_expr_method_call(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        self.check_signature(&node.sig);
        syn::visit::visit_impl_item_fn(self, node);
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        let is_main = node.sig.ident == "main";
        self.startup_depth += usize::from(is_main);
        self.check_signature(&node.sig);
        syn::visit::visit_item_fn(self, node);
        self.startup_depth -= usize::from(is_main);
    }

    fn visit_item_impl(&mut self, node: &'ast syn::ItemImpl) {
        let was = self.in_trait_impl;
        self.in_trait_impl = node.trait_.is_some();
        syn::visit::visit_item_impl(self, node);
        self.in_trait_impl = was;
    }

    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        if self.startup_depth > 0 {
            return;
        }
        let Some(name) = node.path.segments.last().map(|segment| segment.ident.to_string()) else {
            return;
        };
        let message = match name.as_str() {
            "panic" => "`panic!` outside startup and tests: return a `Result` so the caller decides",
            "unreachable" => {
                "`unreachable!` is a bet about control flow; make the impossible state unrepresentable instead"
            }
            "todo" | "unimplemented" => {
                "a placeholder panic shipped to production is a note that the work isn't done"
            }
            _ => return,
        };
        self.findings.report(
            self.cx,
            Rule::PanicInProduction,
            node.path.span(),
            message.to_string(),
        );
    }

    fn visit_trait_item_fn(&mut self, node: &'ast syn::TraitItemFn) {
        self.check_signature(&node.sig);
        syn::visit::visit_trait_item_fn(self, node);
    }
}
