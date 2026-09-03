use std::fmt;

use serde::{Deserialize, Serialize};

use crate::diagnostic::Level;

const BLOG: &str = "https://almaju.github.io/blog/docs";

/// Every check rabot knows about. One rule, one principle, one article.
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Rule {
    CommentedOutCode,
    BypassableConstructor,
    FreeFunction,
    GlobalState,
    IgnoredTest,
    MockUsage,
    OrphanModule,
    OversizedImpl,
    PanicInProduction,
    PrimitiveField,
    PrimitiveSoup,
    SectionedFunction,
    SortedDerives,
    SortedFields,
    SortedImplItems,
    SortedStructLiteral,
    SortedStructPattern,
    SortedTraitItems,
    SortedVariants,
    StringlyTypedField,
    SwallowedError,
    SyntaxError,
    TooManyParameters,
    UndocumentedException,
    UnknownRule,
    UntypedError,
    VagueTodo,
    VagueTypeName,
}

impl Rule {
    pub fn all() -> &'static [Rule] {
        &[
            Rule::CommentedOutCode,
            Rule::BypassableConstructor,
            Rule::FreeFunction,
            Rule::GlobalState,
            Rule::IgnoredTest,
            Rule::MockUsage,
            Rule::OrphanModule,
            Rule::OversizedImpl,
            Rule::PanicInProduction,
            Rule::PrimitiveField,
            Rule::PrimitiveSoup,
            Rule::SectionedFunction,
            Rule::SortedDerives,
            Rule::SortedFields,
            Rule::SortedImplItems,
            Rule::SortedStructLiteral,
            Rule::SortedStructPattern,
            Rule::SortedTraitItems,
            Rule::SortedVariants,
            Rule::StringlyTypedField,
            Rule::SwallowedError,
            Rule::SyntaxError,
            Rule::TooManyParameters,
            Rule::UndocumentedException,
            Rule::UnknownRule,
            Rule::UntypedError,
            Rule::VagueTodo,
            Rule::VagueTypeName,
        ]
    }

    pub fn parse(name: &str) -> Option<Rule> {
        Rule::all().iter().copied().find(|rule| rule.name() == name)
    }

    pub fn default_level(self) -> Level {
        match self {
            Rule::SyntaxError | Rule::UndocumentedException | Rule::UnknownRule => Level::Error,
            _ => Level::Warn,
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Rule::BypassableConstructor => {
                "A newtype that validates in a constructor but exposes its field can be built without it."
            }
            Rule::CommentedOutCode => "Commented-out code. You have git; there is no temporary.",
            Rule::FreeFunction => {
                "A free function whose primary parameter or return type is a local type belongs on that type."
            }
            Rule::GlobalState => "Mutable global state hides dependencies from the type signature.",
            Rule::IgnoredTest => "An ignored test without a reason is a skipped test nobody will un-skip.",
            Rule::MockUsage => "Mocks test your assumptions. Build a real in-memory implementation.",
            Rule::OrphanModule => "A `utils`-style module is where orphaned logic goes to die.",
            Rule::OversizedImpl => {
                "An impl with too many methods is several types that have not been separated yet."
            }
            Rule::PanicInProduction => {
                "`unwrap`, `expect` and `panic!` outside tests and startup are bets that a call never fails."
            }
            Rule::PrimitiveField => {
                "A field named like a domain concept but typed as a primitive accepts anything. Wrap it."
            }
            Rule::PrimitiveSoup => {
                "Several parameters of the same primitive type can be swapped silently. Wrap them."
            }
            Rule::SectionedFunction => {
                "A function narrated by section comments is several functions; the headers are their names."
            }
            Rule::SortedDerives => "Derive lists are sorted alphabetically.",
            Rule::SortedFields => "Struct fields are sorted alphabetically.",
            Rule::SortedImplItems => {
                "Impl items are ordered: consts, types, constructors, pub fns, private fns."
            }
            Rule::SortedStructLiteral => "Struct literal fields are sorted alphabetically.",
            Rule::SortedStructPattern => "Struct pattern fields are sorted alphabetically.",
            Rule::SortedTraitItems => {
                "Trait items are ordered: consts, types, then fns, each alphabetically."
            }
            Rule::SortedVariants => "Enum variants are sorted alphabetically.",
            Rule::StringlyTypedField => "A `status: String` field is an enum that has not been written yet.",
            Rule::SwallowedError => {
                "An empty `Err` arm or a trailing `.ok();` is a silent catch: a future 3am."
            }
            Rule::SyntaxError => "The file does not parse as Rust.",
            Rule::TooManyParameters => {
                "A function with too many parameters is a type that has not been split yet."
            }
            Rule::UndocumentedException => {
                "An allow comment without a reason is chaos with better intentions."
            }
            Rule::UnknownRule => "An allow comment names a rule rabot does not know.",
            Rule::UntypedError => {
                "`Box<dyn Error>` and `anyhow` erase the error taxonomy callers need to decide."
            }
            Rule::VagueTodo => "A TODO without context is noise with a timestamp.",
            Rule::VagueTypeName => "Service, Manager, Handler: names for decisions you have not made yet.",
        }
    }

    /// The rule's page from the documentation, Markdown, for `rabot explain`.
    pub fn documentation(self) -> &'static str {
        macro_rules! page {
            ($name:literal) => {
                include_str!(concat!("../docs/src/rules/", $name, ".md"))
            };
        }
        match self {
            Rule::BypassableConstructor => page!("bypassable-constructor"),
            Rule::CommentedOutCode => page!("commented-out-code"),
            Rule::FreeFunction => page!("free-function"),
            Rule::GlobalState => page!("global-state"),
            Rule::IgnoredTest => page!("ignored-test"),
            Rule::MockUsage => page!("mock-usage"),
            Rule::OrphanModule => page!("orphan-module"),
            Rule::OversizedImpl => page!("oversized-impl"),
            Rule::PanicInProduction => page!("panic-in-production"),
            Rule::PrimitiveField => page!("primitive-field"),
            Rule::PrimitiveSoup => page!("primitive-soup"),
            Rule::SectionedFunction => page!("sectioned-function"),
            Rule::SortedDerives => page!("sorted-derives"),
            Rule::SortedFields => page!("sorted-fields"),
            Rule::SortedImplItems => page!("sorted-impl-items"),
            Rule::SortedStructLiteral => page!("sorted-struct-literal"),
            Rule::SortedStructPattern => page!("sorted-struct-pattern"),
            Rule::SortedTraitItems => page!("sorted-trait-items"),
            Rule::SortedVariants => page!("sorted-variants"),
            Rule::StringlyTypedField => page!("stringly-typed-field"),
            Rule::SwallowedError => page!("swallowed-error"),
            Rule::SyntaxError => page!("syntax-error"),
            Rule::TooManyParameters => page!("too-many-parameters"),
            Rule::UndocumentedException => page!("undocumented-exception"),
            Rule::UnknownRule => page!("unknown-rule"),
            Rule::UntypedError => page!("untyped-error"),
            Rule::VagueTodo => page!("vague-todo"),
            Rule::VagueTypeName => page!("vague-type-name"),
        }
    }

    /// The published page for this rule.
    pub fn documentation_url(self) -> String {
        format!("https://almaju.github.io/rabot/rules/{}.html", self.name())
    }

    pub fn fixable(self) -> bool {
        matches!(
            self,
            Rule::SortedDerives
                | Rule::SortedFields
                | Rule::SortedImplItems
                | Rule::SortedStructLiteral
                | Rule::SortedStructPattern
                | Rule::SortedTraitItems
                | Rule::SortedVariants
        )
    }

    pub fn name(self) -> &'static str {
        match self {
            Rule::BypassableConstructor => "bypassable-constructor",
            Rule::CommentedOutCode => "commented-out-code",
            Rule::FreeFunction => "free-function",
            Rule::GlobalState => "global-state",
            Rule::IgnoredTest => "ignored-test",
            Rule::MockUsage => "mock-usage",
            Rule::OrphanModule => "orphan-module",
            Rule::OversizedImpl => "oversized-impl",
            Rule::PanicInProduction => "panic-in-production",
            Rule::PrimitiveField => "primitive-field",
            Rule::PrimitiveSoup => "primitive-soup",
            Rule::SectionedFunction => "sectioned-function",
            Rule::SortedDerives => "sorted-derives",
            Rule::SortedFields => "sorted-fields",
            Rule::SortedImplItems => "sorted-impl-items",
            Rule::SortedStructLiteral => "sorted-struct-literal",
            Rule::SortedStructPattern => "sorted-struct-pattern",
            Rule::SortedTraitItems => "sorted-trait-items",
            Rule::SortedVariants => "sorted-variants",
            Rule::StringlyTypedField => "stringly-typed-field",
            Rule::SwallowedError => "swallowed-error",
            Rule::SyntaxError => "syntax-error",
            Rule::TooManyParameters => "too-many-parameters",
            Rule::UndocumentedException => "undocumented-exception",
            Rule::UnknownRule => "unknown-rule",
            Rule::UntypedError => "untyped-error",
            Rule::VagueTodo => "vague-todo",
            Rule::VagueTypeName => "vague-type-name",
        }
    }

    /// The article that states the principle behind this rule.
    pub fn reference(self) -> String {
        let page = match self {
            Rule::CommentedOutCode | Rule::SectionedFunction | Rule::VagueTodo => {
                "fundamentals/style/comments"
            }
            Rule::FreeFunction | Rule::VagueTypeName => "fundamentals/modeling/method-ownership",
            Rule::GlobalState => "fundamentals/architecture/dependencies",
            Rule::IgnoredTest | Rule::MockUsage => "fundamentals/architecture/testing",
            Rule::OrphanModule | Rule::OversizedImpl | Rule::TooManyParameters => {
                "fundamentals/modeling/structs"
            }
            Rule::PanicInProduction | Rule::SwallowedError | Rule::UntypedError => {
                "fundamentals/modeling/errors"
            }
            Rule::BypassableConstructor
            | Rule::PrimitiveField
            | Rule::PrimitiveSoup
            | Rule::StringlyTypedField => "fundamentals/modeling/primitives",
            Rule::SortedDerives
            | Rule::SortedFields
            | Rule::SortedImplItems
            | Rule::SortedStructLiteral
            | Rule::SortedStructPattern
            | Rule::SortedTraitItems
            | Rule::SortedVariants
            | Rule::SyntaxError
            | Rule::UndocumentedException
            | Rule::UnknownRule => "fundamentals/style/sorting",
        };
        format!("{BLOG}/{page}")
    }
}

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_rule_is_documented_with_dos_and_donts() {
        for rule in Rule::all() {
            let page = rule.documentation();
            assert!(page.starts_with(&format!("# {}\n", rule.name())), "{rule}: title");
            assert!(page.contains("## What it checks"), "{rule}: what it checks");
            if !matches!(rule, Rule::SyntaxError) {
                assert!(
                    page.contains("## Don't") && page.contains("## Do"),
                    "{rule}: do and don't"
                );
            }
            assert!(
                page.contains("**Level**: error") || page.contains("**Level**: warn"),
                "{rule}: level"
            );
        }
    }

    #[test]
    fn documented_levels_match_defaults() {
        for rule in Rule::all() {
            let expected = format!("**Level**: {}", rule.default_level());
            let page = rule.documentation();
            let stated = if page.contains("**Level**: error") {
                "**Level**: error"
            } else {
                "**Level**: warn"
            };
            assert_eq!(stated, expected.replace("warning", "warn"), "{rule}");
        }
    }

    #[test]
    fn names_round_trip() {
        for rule in Rule::all() {
            assert_eq!(Rule::parse(rule.name()), Some(*rule));
        }
    }
}
