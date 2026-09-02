use std::io::{self, IsTerminal, Write};
use std::path::Path;

use crate::app::Outcome;
use crate::diagnostic::{Diagnostic, Level};

/// How findings reach the user.
#[derive(clap::ValueEnum, Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Format {
    /// One JSON array of diagnostics.
    Json,
    /// rustc-style text.
    #[default]
    Text,
}

/// How a piece of output is painted when the terminal supports colour.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
    format: Format,
    root: &'a Path,
}

impl<'a> Report<'a> {
    pub fn new(format: Format, root: &'a Path) -> Self {
        Self {
            color: format == Format::Text && io::stdout().is_terminal(),
            format,
            root,
        }
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
            "  {} see: {}",
            self.paint(Style::Location, "="),
            diagnostic.rule.reference()
        )?;
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
            "changed": outcome.changed.iter().map(|path| self.relative(path)).collect::<Vec<_>>(),
            "diagnostics": diagnostics,
            "files_seen": outcome.files_seen,
        });
        writeln!(out, "{}", serde_json::to_string_pretty(&body).unwrap_or_default())
    }

    fn write_text(&self, outcome: &Outcome, out: &mut dyn Write) -> io::Result<()> {
        for diagnostic in &outcome.diagnostics {
            self.write_diagnostic(diagnostic, out)?;
        }
        for path in &outcome.changed {
            writeln!(
                out,
                "{} {}",
                self.paint(Style::Success, "reordered"),
                self.relative(path)
            )?;
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
