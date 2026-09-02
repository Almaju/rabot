//! Repository, Handler, and Service are names for decisions you haven't made yet.
//! <https://almaju.github.io/blog/docs/fundamentals/modeling/method-ownership>

use syn::visit::Visit;

use crate::rule::Rule;
use crate::rules::{Check, Context, Findings};

pub struct Naming;

impl Check for Naming {
    fn run(&self, cx: &Context) -> Findings {
        let mut visitor = Visitor {
            cx,
            findings: Findings::default(),
        };
        visitor.visit_file(&cx.file.ast);
        visitor.findings
    }
}

struct Visitor<'a> {
    cx: &'a Context<'a>,
    findings: Findings,
}

impl Visitor<'_> {
    fn check_type_name(&mut self, kind: &str, ident: &syn::Ident) {
        let name = ident.to_string();
        let suffix = self
            .cx
            .config
            .naming
            .vague_suffixes
            .iter()
            .find(|suffix| WordSuffix(suffix).ends(&name));
        let Some(suffix) = suffix else {
            return;
        };
        let subject = name.strip_suffix(suffix.as_str()).unwrap_or("");
        let help = if subject.is_empty() {
            "name it after what it is, not after the pattern you read about".to_string()
        } else {
            format!(
                "the behaviour probably belongs on `{subject}` itself; if something must orchestrate, name what it orchestrates"
            )
        };
        self.findings.report_with_help(
            self.cx,
            Rule::VagueTypeName,
            ident.span(),
            format!("{kind} `{name}` ends in `{suffix}`: a name for a decision that has not been made yet"),
            Some(help),
        );
    }
}

impl<'ast> Visit<'ast> for Visitor<'_> {
    fn visit_item_enum(&mut self, node: &'ast syn::ItemEnum) {
        self.check_type_name("enum", &node.ident);
    }

    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        let name = node.ident.to_string();
        if self.cx.config.naming.orphan_modules.contains(&name) {
            self.findings.report_with_help(
                self.cx,
                Rule::OrphanModule,
                node.ident.span(),
                format!("module `{name}` is a drawer: every function in it is a method on a type that does not exist yet"),
                Some("find the parameters those functions share, name that struct, and move them onto it".to_string()),
            );
        }
        syn::visit::visit_item_mod(self, node);
    }

    fn visit_item_struct(&mut self, node: &'ast syn::ItemStruct) {
        self.check_type_name("struct", &node.ident);
    }

    fn visit_item_trait(&mut self, node: &'ast syn::ItemTrait) {
        self.check_type_name("trait", &node.ident);
    }

    fn visit_item_type(&mut self, node: &'ast syn::ItemType) {
        self.check_type_name("type alias", &node.ident);
    }
}

/// A suffix that must be a whole CamelCase word: `RequestHandler` ends with
/// the word `Handler`; `Chandler` does not.
struct WordSuffix<'a>(&'a str);

impl WordSuffix<'_> {
    fn ends(&self, name: &str) -> bool {
        let Some(prefix) = name.strip_suffix(self.0) else {
            return false;
        };
        prefix.is_empty()
            || !prefix.ends_with(|c: char| c.is_ascii_lowercase())
            || self.0.starts_with(|c: char| c.is_ascii_uppercase())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suffix_must_be_a_word() {
        assert!(WordSuffix("Service").ends("UserService"));
        assert!(WordSuffix("Service").ends("Service"));
        assert!(WordSuffix("Utils").ends("Utils"));
        assert!(!WordSuffix("Handler").ends("Chandler"));
    }
}
