use std::fmt;

use serde::{Deserialize, Serialize};

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
