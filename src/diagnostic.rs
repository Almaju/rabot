use std::path::PathBuf;

use serde::Serialize;

pub use crate::level::Level;
use crate::rule::Rule;

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
