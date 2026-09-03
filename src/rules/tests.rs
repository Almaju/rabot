//! Your tests are either fast or honest. Real in-memory implementations are
//! both. Mocks are neither.
//! <https://almaju.github.io/blog/docs/fundamentals/architecture/testing>

use syn::spanned::Spanned;
use syn::visit::Visit;

use crate::rule::Rule;
use crate::rules::{Check, Context, Findings};

pub struct Tests;

impl Check for Tests {
    fn run(&self, cx: &Context) -> Findings {
        let mut visitor = Visitor {
            cx,
            findings: Findings::default(),
        };
        visitor.visit_file(&cx.file.ast);
        visitor.findings
    }
}

const MOCK_CRATES: [&str; 5] = ["faux", "mockall", "mockall_double", "mry", "unimock"];
const HELP: &str = "write the in-memory implementation (a `MemDatabase` with a `HashMap` that enforces the same constraints); it earns its place in the codebase";

struct Visitor<'a> {
    cx: &'a Context<'a>,
    findings: Findings,
}

impl<'ast> Visit<'ast> for Visitor<'_> {
    fn visit_attribute(&mut self, node: &'ast syn::Attribute) {
        if node.path().is_ident("ignore") && matches!(node.meta, syn::Meta::Path(_)) {
            self.findings.report_with_help(
                self.cx,
                Rule::IgnoredTest,
                node.span(),
                "`#[ignore]` without a reason: in six months nobody will know why this test is skipped, or whether it may run again"
                    .to_string(),
                Some("write it down: `#[ignore = \"flaky on CI since PERF-112, see ...\"]`".to_string()),
            );
        }
        let text = self.cx.file.text_of(node);
        if text.contains("automock") || text.contains("mockall::") || text.contains("faux::") {
            self.findings.report_with_help(
                self.cx,
                Rule::MockUsage,
                node.span(),
                "a generated mock does exactly what you tell it to; that is the problem".to_string(),
                Some(HELP.to_string()),
            );
        }
    }

    fn visit_item_use(&mut self, node: &'ast syn::ItemUse) {
        let root = match &node.tree {
            syn::UseTree::Path(path) => path.ident.to_string(),
            syn::UseTree::Name(name) => name.ident.to_string(),
            _ => return,
        };
        if MOCK_CRATES.contains(&root.as_str()) {
            self.findings.report_with_help(
                self.cx,
                Rule::MockUsage,
                node.span(),
                format!("`{root}` mocks test your assumptions, not your code"),
                Some(HELP.to_string()),
            );
        }
    }

    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        let is_mock = node
            .path
            .segments
            .last()
            .is_some_and(|segment| segment.ident == "mock");
        if is_mock {
            self.findings.report_with_help(
                self.cx,
                Rule::MockUsage,
                node.path.span(),
                "`mock!` builds a self-fulfilling prophecy: it returns what the test needs, never what the dependency would".to_string(),
                Some(HELP.to_string()),
            );
        }
    }
}
