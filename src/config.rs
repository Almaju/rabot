use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::diagnostic::Level;
use crate::rule::Rule;

pub const FILE_NAME: &str = "rabot.toml";

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("cannot parse {path}: {source}")]
    Parse {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
    #[error("cannot read {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

/// Everything a project can tune, read from `rabot.toml`.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct Config {
    pub files: Files,
    pub global_state: GlobalState,
    pub naming: Naming,
    /// Per-rule level overrides. Rules not listed keep their default.
    pub rules: BTreeMap<Rule, Level>,
    pub sorting: Sorting,
    pub tests: Tests,
    pub thresholds: Thresholds,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct Tests {
    /// Rules that stay silent in test code: `#[cfg(test)]` items, `#[test]`
    /// functions, and files under `tests/`, `benches/` or `examples/`.
    /// Sorting rules and the comment rules are not relaxed by default: a
    /// test file is still code.
    pub relax: Vec<Rule>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct Sorting {
    /// Derives pinned to a position. Names before `...` come first in the
    /// given order, names after it come last; everything else sits in
    /// between, alphabetically, with each derive after the trait it extends
    /// (`Eq` after `PartialEq`, `Ord` after `PartialOrd`, `Copy` after
    /// `Clone`). Empty means: no pins, only that rule.
    pub derive_order: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct Files {
    /// Glob patterns (gitignore syntax) of paths to skip.
    pub exclude: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct GlobalState {
    /// Statics whose name contains one of these (case-insensitive) are
    /// infrastructure, not hidden dependencies. A logger is the classic case.
    pub allowed_names: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct Naming {
    /// Type-name suffixes that mark a wire shape (a request body, a database
    /// row): primitives are expected there, parsing happens right after.
    pub boundary_suffixes: Vec<String>,
    /// Field names (or `_`-suffixes such as `_id`) that carry domain meaning
    /// and deserve a type of their own.
    pub domain_fields: Vec<String>,
    /// Field names whose `String` value is really one of a few variants.
    pub enum_fields: Vec<String>,
    /// Variant names of an error enum that catch whatever nobody named.
    pub escape_hatch_variants: Vec<String>,
    /// Module names that collect orphaned logic.
    pub orphan_modules: Vec<String>,
    /// Type-name suffixes that stand in for a decision nobody made.
    pub vague_suffixes: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct Thresholds {
    /// Methods (across inherent impls in one file) before an impl is oversized.
    pub oversized_impl: usize,
    /// Parameters of the same primitive type before a signature is soup.
    pub primitive_soup: usize,
    /// Leading comments inside one function body before it reads as a
    /// table of contents.
    pub section_comments: usize,
    /// Parameters (excluding `self`) before a function has too many.
    pub too_many_parameters: usize,
    /// Words a TODO needs, unless it references a ticket or URL.
    pub vague_todo_min_words: usize,
}

impl Config {
    /// Read `rabot.toml` from `root`, or the defaults when there is none.
    pub fn load(root: &Path) -> Result<Self, ConfigError> {
        let path = root.join(FILE_NAME);
        if !path.exists() {
            return Ok(Self::default());
        }
        Self::read(&path)
    }

    pub fn read(path: &Path) -> Result<Self, ConfigError> {
        let text = std::fs::read_to_string(path).map_err(|source| ConfigError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        toml::from_str(&text).map_err(|source| ConfigError::Parse {
            path: path.to_path_buf(),
            source,
        })
    }

    pub fn level(&self, rule: Rule) -> Level {
        self.rules.get(&rule).copied().unwrap_or(rule.default_level())
    }

    /// The default configuration as TOML, for `rabot init`.
    pub fn template() -> String {
        let mut config = Self::default();
        for rule in Rule::all() {
            config.rules.insert(*rule, rule.default_level());
        }
        let body = toml::to_string_pretty(&config).unwrap_or_default();
        format!(
            "# rabot configuration. Every rule maps to an article on\n# https://almaju.github.io/blog/\n#\n# Levels: \"allow\", \"warn\", \"error\".\n\n{body}"
        )
    }
}

impl Default for Files {
    fn default() -> Self {
        Self {
            exclude: vec!["target".to_string()],
        }
    }
}

impl Default for GlobalState {
    fn default() -> Self {
        Self {
            allowed_names: vec!["LOG".to_string()],
        }
    }
}

impl Default for Naming {
    fn default() -> Self {
        Self {
            boundary_suffixes: [
                "Body", "Dto", "Params", "Payload", "Query", "Record", "Request", "Response", "Row",
            ]
            .map(str::to_string)
            .to_vec(),
            domain_fields: [
                "_id",
                "amount",
                "currency",
                "email",
                "hash",
                "iban",
                "id",
                "lat",
                "latitude",
                "lng",
                "lon",
                "longitude",
                "password",
                "percent",
                "percentage",
                "phone",
                "price",
                "secret",
                "slug",
                "timestamp",
                "token",
                "uri",
                "url",
                "zip",
            ]
            .map(str::to_string)
            .to_vec(),
            enum_fields: [
                "category", "kind", "level", "mode", "phase", "role", "stage", "state", "status", "ty",
                "type_",
            ]
            .map(str::to_string)
            .to_vec(),
            escape_hatch_variants: [
                "Custom",
                "Generic",
                "Internal",
                "Misc",
                "Other",
                "Unexpected",
                "Unknown",
            ]
            .map(str::to_string)
            .to_vec(),
            orphan_modules: ["common", "helper", "helpers", "misc", "util", "utils"]
                .map(str::to_string)
                .to_vec(),
            vague_suffixes: [
                "Controller",
                "Coordinator",
                "Handler",
                "Helper",
                "Manager",
                "Processor",
                "Repository",
                "Service",
                "UseCase",
                "Util",
                "Utils",
            ]
            .map(str::to_string)
            .to_vec(),
        }
    }
}

impl Default for Tests {
    fn default() -> Self {
        Self {
            relax: vec![
                Rule::AmbientConfig,
                Rule::AmbientRandomness,
                Rule::AmbientTime,
                Rule::BooleanValidation,
                Rule::BypassableConstructor,
                Rule::DroppedErrorContext,
                Rule::FreeFunction,
                Rule::GlobalState,
                Rule::OrphanModule,
                Rule::OversizedImpl,
                Rule::PanicInProduction,
                Rule::PrimitiveField,
                Rule::PrimitiveSoup,
                Rule::SectionedFunction,
                Rule::StringlyTypedField,
                Rule::SwallowedError,
                Rule::TooManyParameters,
                Rule::UntypedError,
                Rule::VagueTypeName,
            ],
        }
    }
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            oversized_impl: 20,
            primitive_soup: 2,
            section_comments: 3,
            too_many_parameters: 7,
            vague_todo_min_words: 6,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn template_round_trips() {
        let config: Config = toml::from_str(&Config::template()).unwrap();
        assert_eq!(config.level(Rule::SortedFields), Level::Warn);
        assert_eq!(config.thresholds.oversized_impl, 20);
    }

    #[test]
    fn overrides_levels() {
        let config: Config = toml::from_str("[rules]\nfree-function = \"allow\"\n").unwrap();
        assert_eq!(config.level(Rule::FreeFunction), Level::Allow);
        assert_eq!(config.level(Rule::VagueTodo), Level::Warn);
    }

    #[test]
    fn rejects_unknown_keys() {
        assert!(toml::from_str::<Config>("[rules]\nnope = \"allow\"\n").is_err());
        assert!(toml::from_str::<Config>("[typo]\n").is_err());
    }
}
