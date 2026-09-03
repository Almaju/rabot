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

const SWALLOW_HELP: &str =
    "propagate with `?`, log it with the context a reader at 3am needs, or match on the variant and act";

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
        let (erased, why) = if text.contains("dyn") && text.contains("Error") {
            (
                "`Box<dyn Error>`",
                "callers cannot match on what went wrong, so they will guess",
            )
        } else if text.contains("anyhow") {
            (
                "`anyhow`",
                "callers cannot match on what went wrong, so they will guess",
            )
        } else if text.contains("eyre") {
            (
                "`eyre`",
                "callers cannot match on what went wrong, so they will guess",
            )
        } else if string_error(ty) {
            ("a `String` error", "callers can display it, never react to it")
        } else {
            return;
        };
        self.findings.report_with_help(
            self.cx,
            Rule::UntypedError,
            ty.span(),
            format!("`{}` returns {erased}: {why}", sig.ident),
            Some("return an enum of the failures a caller can react to differently".to_string()),
        );
    }
}

impl<'ast> Visit<'ast> for Visitor<'_> {
    fn visit_arm(&mut self, node: &'ast syn::Arm) {
        if is_err_pattern(&node.pat) && is_empty(&node.body) {
            self.findings.report_with_help(
                self.cx,
                Rule::SwallowedError,
                node.pat.span(),
                "an empty `Err` arm is a silent catch: the failure happened and nobody will know".to_string(),
                Some(SWALLOW_HELP.to_string()),
            );
        }
        syn::visit::visit_arm(self, node);
    }

    fn visit_expr_if(&mut self, node: &'ast syn::ExprIf) {
        if let syn::Expr::Let(let_expr) = &*node.cond
            && is_err_pattern(&let_expr.pat)
            && node.then_branch.stmts.is_empty()
            && node.else_branch.is_none()
        {
            self.findings.report_with_help(
                self.cx,
                Rule::SwallowedError,
                let_expr.pat.span(),
                "`if let Err(..)` with an empty body is a silent catch: the failure happened and nobody will know"
                    .to_string(),
                Some(SWALLOW_HELP.to_string()),
            );
        }
        syn::visit::visit_expr_if(self, node);
    }

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

    fn visit_stmt(&mut self, node: &'ast syn::Stmt) {
        if let syn::Stmt::Expr(syn::Expr::MethodCall(call), Some(_)) = node
            && call.method == "ok"
            && call.args.is_empty()
        {
            self.findings.report_with_help(
                self.cx,
                Rule::SwallowedError,
                call.method.span(),
                "`.ok();` as a statement throws the error away".to_string(),
                Some(SWALLOW_HELP.to_string()),
            );
        }
        syn::visit::visit_stmt(self, node);
    }

    fn visit_trait_item_fn(&mut self, node: &'ast syn::TraitItemFn) {
        self.check_signature(&node.sig);
        syn::visit::visit_trait_item_fn(self, node);
    }
}

fn is_empty(body: &syn::Expr) -> bool {
    match body {
        syn::Expr::Block(block) => block.block.stmts.is_empty(),
        syn::Expr::Tuple(tuple) => tuple.elems.is_empty(),
        _ => false,
    }
}

fn is_err_pattern(pat: &syn::Pat) -> bool {
    match pat {
        syn::Pat::TupleStruct(pattern) => pattern
            .path
            .segments
            .last()
            .is_some_and(|segment| segment.ident == "Err"),
        _ => false,
    }
}

/// `Result<T, String>`, `Result<T, &str>`, `Result<T, &'static str>`.
fn string_error(ty: &syn::Type) -> bool {
    let syn::Type::Path(path) = ty else {
        return false;
    };
    let Some(segment) = path.path.segments.last() else {
        return false;
    };
    if segment.ident != "Result" {
        return false;
    }
    let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return false;
    };
    let Some(syn::GenericArgument::Type(error)) = arguments.args.iter().nth(1) else {
        return false;
    };
    match error {
        syn::Type::Path(error) => error.path.is_ident("String"),
        syn::Type::Reference(reference) => {
            matches!(&*reference.elem, syn::Type::Path(inner) if inner.path.is_ident("str"))
        }
        _ => false,
    }
}
