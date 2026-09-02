//! rabot: a linter and formatter that enforces the principles of
//! <https://almaju.github.io/blog/> on Rust code.
//!
//! Every rule maps to one article. Every exception must be written down.

pub mod allowance;
pub mod app;
pub mod cli;
pub mod comment;
pub mod config;
pub mod diagnostic;
pub mod edit;
pub mod file_set;
pub mod hook;
pub mod ordering;
pub mod report;
pub mod rule;
pub mod rules;
pub mod rustfmt;
pub mod source_file;
pub mod test_regions;
