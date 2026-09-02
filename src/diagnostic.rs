use std::fmt;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::rule::Rule;

/// How loudly a rule speaks.
///
/// Variant order is semantic (`Allow < Warn < Error`) so it can be compared.
// rabot: allow(sorted-variants) ordering is semantic and used for comparisons
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Allow,
    Warn,
    Error,
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Level::Allow => "allow",
            Level::Error => "error",
            Level::Warn => "warning",
        })
    }
}

/// A 1-based line and column in a source file.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct Position {
    pub column: usize,
    pub line: usize,
}

/// One finding, ready to be shown to a human or a machine.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Diagnostic {
    pub help: Option<String>,
    pub level: Level,
    pub message: String,
    pub path: PathBuf,
    pub position: Position,
    pub rule: Rule,
}

impl Diagnostic {
    pub fn sort_key(&self) -> (PathBuf, Position, Rule) {
        (self.path.clone(), self.position, self.rule)
    }
}
