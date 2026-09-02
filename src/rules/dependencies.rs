//! If it's not in the signature, it shouldn't exist.
//! <https://almaju.github.io/blog/docs/fundamentals/architecture/dependencies>

use syn::spanned::Spanned;
use syn::visit::Visit;

use crate::rule::Rule;
use crate::rules::{Check, Context, Findings};

pub struct Dependencies;

impl Check for Dependencies {
    fn run(&self, cx: &Context) -> Findings {
        let mut visitor = Visitor {
            cx,
            findings: Findings::default(),
        };
        visitor.visit_file(&cx.file.ast);
        visitor.findings
    }
}

const INTERIOR_MUTABILITY: [&str; 10] = [
    "Atomic",
    "Cell",
    "Lazy",
    "LazyLock",
    "Mutex",
    "OnceCell",
    "OnceLock",
    "RefCell",
    "RwLock",
    "UnsafeCell",
];

struct Visitor<'a> {
    cx: &'a Context<'a>,
    findings: Findings,
}

impl Visitor<'_> {
    fn is_infrastructure(&self, name: &str) -> bool {
        let upper = name.to_ascii_uppercase();
        self.cx
            .config
            .global_state
            .allowed_names
            .iter()
            .any(|allowed| upper.contains(&allowed.to_ascii_uppercase()))
    }
}

impl<'ast> Visit<'ast> for Visitor<'_> {
    fn visit_item_macro(&mut self, node: &'ast syn::ItemMacro) {
        let is_lazy_static = node
            .mac
            .path
            .segments
            .last()
            .is_some_and(|segment| segment.ident == "lazy_static");
        if is_lazy_static {
            self.findings.report(
                self.cx,
                Rule::GlobalState,
                node.span(),
                "`lazy_static!` creates global state; dependencies belong in the type signature, wired once in `main`"
                    .to_string(),
            );
        }
    }

    fn visit_item_static(&mut self, node: &'ast syn::ItemStatic) {
        let name = node.ident.to_string();
        if self.is_infrastructure(&name) {
            return;
        }
        let type_text = self.cx.file.text_of(&node.ty);
        let mutable = matches!(node.mutability, syn::StaticMutability::Mut(_))
            || INTERIOR_MUTABILITY
                .iter()
                .any(|marker| type_text.contains(marker));
        if !mutable {
            return;
        }
        self.findings.report_with_help(
            self.cx,
            Rule::GlobalState,
            node.ident.span(),
            format!("`{name}` is global mutable state: a dependency hidden from every signature that uses it"),
            Some(
                "pass it as a constructor or method parameter; if it is genuinely infrastructure nobody swaps in tests, add its name to `[global-state] allowed-names`"
                    .to_string(),
            ),
        );
    }
}
