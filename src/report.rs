use std::io::{self, IsTerminal, Write};
use std::path::Path;

use similar::{ChangeTag, TextDiff};

use crate::app::{Change, Outcome};
use crate::diagnostic::{Diagnostic, Level};

/// How findings reach the user.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum Format {
    /// One JSON array of diagnostics.
    Json,
    /// rustc-style text.
    #[default]
    Text,
}

/// How a piece of output is painted when the terminal supports colour.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Style {
    Bold,
    Error,
    Location,
    Success,
    Warning,
}

impl Style {
    fn code(self) -> &'static str {
        match self {
            Style::Bold => "1",
            Style::Error => "1;31",
            Style::Location => "1;34",
            Style::Success => "1;32",
            Style::Warning => "1;33",
        }
    }
}

/// Turns an [`Outcome`] into text on a writer.
pub struct Report<'a> {
    color: bool,
    diff: bool,
    format: Format,
    root: &'a Path,
}

impl<'a> Report<'a> {
    pub fn new(format: Format, root: &'a Path) -> Self {
        Self {
            color: format == Format::Text && io::stdout().is_terminal(),
            diff: false,
            format,
            root,
        }
    }

    /// Show a unified diff for every file `fmt` changed or would change.
    pub fn with_diff(mut self, diff: bool) -> Self {
        self.diff = diff;
        self
    }

    pub fn write(&self, outcome: &Outcome, out: &mut dyn Write) -> io::Result<()> {
        match self.format {
            Format::Json => self.write_json(outcome, out),
            Format::Text => self.write_text(outcome, out),
        }
    }

    fn paint(&self, style: Style, text: &str) -> String {
        if self.color {
            format!("\x1b[{}m{text}\x1b[0m", style.code())
        } else {
            text.to_string()
        }
    }

    fn relative(&self, path: &Path) -> String {
        path.strip_prefix(self.root).unwrap_or(path).display().to_string()
    }

    fn write_diagnostic(&self, diagnostic: &Diagnostic, out: &mut dyn Write) -> io::Result<()> {
        let (style, label) = match diagnostic.level {
            Level::Error => (Style::Error, "error"),
            Level::Warn => (Style::Warning, "warning"),
            Level::Allow => (Style::Bold, "note"),
        };
        writeln!(
            out,
            "{}{}: {}",
            self.paint(style, label),
            self.paint(Style::Bold, &format!("[{}]", diagnostic.rule)),
            self.paint(Style::Bold, &diagnostic.message)
        )?;
        writeln!(
            out,
            "  {} {}:{}:{}",
            self.paint(Style::Location, "-->"),
            self.relative(&diagnostic.path),
            diagnostic.position.line,
            diagnostic.position.column
        )?;
        if let Some(help) = &diagnostic.help {
            writeln!(out, "  {} help: {help}", self.paint(Style::Location, "="))?;
        }
        writeln!(
            out,
            "  {} docs: rabot explain {} | {}",
            self.paint(Style::Location, "="),
            diagnostic.rule,
            diagnostic.rule.reference()
        )?;
        writeln!(out)
    }

    fn write_diff(&self, change: &Change, path: &str, out: &mut dyn Write) -> io::Result<()> {
        let diff = TextDiff::from_lines(&change.before, &change.after);
        writeln!(
            out,
            "{}",
            self.paint(Style::Bold, &format!("--- {path}\n+++ {path} (rabot fmt)"))
        )?;
        for hunk in diff.unified_diff().context_radius(3).iter_hunks() {
            writeln!(out, "{}", self.paint(Style::Location, &hunk.header().to_string()))?;
            for change in hunk.iter_changes() {
                let (style, sign) = match change.tag() {
                    ChangeTag::Delete => (Style::Error, "-"),
                    ChangeTag::Equal => (Style::Bold, " "),
                    ChangeTag::Insert => (Style::Success, "+"),
                };
                let line = change.value().trim_end_matches('\n');
                let text = format!("{sign}{line}");
                let painted = if change.tag() == ChangeTag::Equal {
                    text
                } else {
                    self.paint(style, &text)
                };
                writeln!(out, "{painted}")?;
            }
        }
        writeln!(out)
    }

    fn write_json(&self, outcome: &Outcome, out: &mut dyn Write) -> io::Result<()> {
        let diagnostics: Vec<serde_json::Value> = outcome
            .diagnostics
            .iter()
            .map(|diagnostic| {
                serde_json::json!({
                    "column": diagnostic.position.column,
                    "help": diagnostic.help,
                    "level": diagnostic.level,
                    "line": diagnostic.position.line,
                    "message": diagnostic.message,
                    "path": self.relative(&diagnostic.path),
                    "reference": diagnostic.rule.reference(),
                    "rule": diagnostic.rule,
                })
            })
            .collect();
        let body = serde_json::json!({
            "changed": outcome.changed.iter().map(|change| self.relative(&change.path)).collect::<Vec<_>>(),
            "diagnostics": diagnostics,
            "files_seen": outcome.files_seen,
        });
        writeln!(out, "{}", serde_json::to_string_pretty(&body).unwrap_or_default())
    }

    fn write_text(&self, outcome: &Outcome, out: &mut dyn Write) -> io::Result<()> {
        for diagnostic in &outcome.diagnostics {
            // In diff mode the diff is the report for anything fmt can fix.
            if self.diff && diagnostic.rule.fixable() {
                continue;
            }
            self.write_diagnostic(diagnostic, out)?;
        }
        for change in &outcome.changed {
            let path = self.relative(&change.path);
            if self.diff {
                self.write_diff(change, &path, out)?;
            } else {
                writeln!(out, "{} {path}", self.paint(Style::Success, "reordered"))?;
            }
        }
        let errors = outcome.count(Level::Error);
        let warnings = outcome.count(Level::Warn);
        let summary = format!(
            "{} file{} checked, {errors} error{}, {warnings} warning{}",
            outcome.files_seen,
            plural(outcome.files_seen),
            plural(errors),
            plural(warnings),
        );
        writeln!(out, "{}", self.paint(Style::Bold, &summary))?;
        if warnings + errors > 0 {
            writeln!(
                out,
                "every exception must be written down: `// rabot: allow(rule-name) reason`"
            )?;
        }
        Ok(())
    }
}

fn plural(count: usize) -> &'static str {
    if count == 1 { "" } else { "s" }
}
