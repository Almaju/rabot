use std::fmt;

use serde::{Deserialize, Serialize};

use crate::diagnostic::Level;

const BLOG: &str = "https://almaju.github.io/blog/docs";

/// Every check rabot knows about. One rule, one principle, one article.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Rule {
    CommentedOutCode,
    FreeFunction,
    GlobalState,
    MockUsage,
    OrphanModule,
    OversizedImpl,
    PanicInProduction,
    PrimitiveField,
    PrimitiveSoup,
    SortedDerives,
    SortedFields,
    SortedImplItems,
    SortedStructLiteral,
    SortedStructPattern,
    SortedTraitItems,
    SortedVariants,
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
            Rule::FreeFunction,
            Rule::GlobalState,
            Rule::MockUsage,
            Rule::OrphanModule,
            Rule::OversizedImpl,
            Rule::PanicInProduction,
            Rule::PrimitiveField,
            Rule::PrimitiveSoup,
            Rule::SortedDerives,
            Rule::SortedFields,
            Rule::SortedImplItems,
            Rule::SortedStructLiteral,
            Rule::SortedStructPattern,
            Rule::SortedTraitItems,
            Rule::SortedVariants,
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
            Rule::CommentedOutCode => "Commented-out code. You have git; there is no temporary.",
            Rule::FreeFunction => {
                "A free function whose primary parameter or return type is a local type belongs on that type."
            }
            Rule::GlobalState => "Mutable global state hides dependencies from the type signature.",
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
            Rule::CommentedOutCode => "commented-out-code",
            Rule::FreeFunction => "free-function",
            Rule::GlobalState => "global-state",
            Rule::MockUsage => "mock-usage",
            Rule::OrphanModule => "orphan-module",
            Rule::OversizedImpl => "oversized-impl",
            Rule::PanicInProduction => "panic-in-production",
            Rule::PrimitiveField => "primitive-field",
            Rule::PrimitiveSoup => "primitive-soup",
            Rule::SortedDerives => "sorted-derives",
            Rule::SortedFields => "sorted-fields",
            Rule::SortedImplItems => "sorted-impl-items",
            Rule::SortedStructLiteral => "sorted-struct-literal",
            Rule::SortedStructPattern => "sorted-struct-pattern",
            Rule::SortedTraitItems => "sorted-trait-items",
            Rule::SortedVariants => "sorted-variants",
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
            Rule::CommentedOutCode | Rule::VagueTodo => "fundamentals/style/comments",
            Rule::FreeFunction | Rule::VagueTypeName => "fundamentals/modeling/method-ownership",
            Rule::GlobalState => "fundamentals/architecture/dependencies",
            Rule::MockUsage => "fundamentals/architecture/testing",
            Rule::OrphanModule | Rule::OversizedImpl | Rule::TooManyParameters => {
                "fundamentals/modeling/structs"
            }
            Rule::PanicInProduction | Rule::UntypedError => "fundamentals/modeling/errors",
            Rule::PrimitiveField | Rule::PrimitiveSoup => "fundamentals/modeling/primitives",
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
