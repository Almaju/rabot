//! The command line. `main.rs` and `cargo-rabot` are thin wrappers over
//! [`Cli::main`].

use std::ffi::OsString;
use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

use crate::app::{App, AppError, FormatMode, Outcome};
use crate::config::{Config, FILE_NAME};
use crate::diagnostic::Level;
use crate::file_set::Scope;
use crate::hook::{HookError, PreCommitHook};
use crate::report::{Format, Report};
use crate::rule::Rule;
use crate::rustfmt::Rustfmt;

/// A linter and formatter for Rust that enforces the principles of
/// https://almaju.github.io/blog/ : sort everything, name what you built,
/// wrap your primitives, treat errors as data, write down every exception.
///
/// With no subcommand, `rabot` runs `check` on the current directory.
#[derive(Debug, Parser)]
#[command(name = "rabot", version, about)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
    /// Directory holding rabot.toml (defaults to the current directory).
    #[arg(long, global = true, default_value = ".")]
    root: PathBuf,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Report everything that breaks a principle. Writes nothing. (default)
    Check {
        /// Only files git sees as changed: uncommitted by default, or since
        /// a ref (`--changed=main`). Migrate on contact.
        #[arg(long, value_name = "REF", num_args = 0..=1, default_missing_value = "HEAD")]
        changed: Option<String>,
        /// Output format.
        #[arg(long, value_enum, default_value_t = Format::Text)]
        format: Format,
        /// Files or directories (defaults to the root).
        paths: Vec<PathBuf>,
        /// Exit non-zero on warnings too.
        #[arg(long)]
        strict: bool,
    },
    /// Sort fields, variants, impl items, derives, struct literals and
    /// patterns in place, then run rustfmt on what changed.
    Fmt {
        /// Only files git sees as changed: uncommitted by default, or since
        /// a ref (`--changed=main`). Migrate on contact.
        #[arg(long, value_name = "REF", num_args = 0..=1, default_missing_value = "HEAD")]
        changed: Option<String>,
        /// Do not write; exit non-zero if any file would change.
        #[arg(long)]
        check: bool,
        /// Print a unified diff of every change (implies --check).
        #[arg(long)]
        diff: bool,
        /// Output format.
        #[arg(long, value_enum, default_value_t = Format::Text)]
        format: Format,
        /// Leave indentation alone: do not run rustfmt on the files rabot rewrote.
        #[arg(long)]
        no_rustfmt: bool,
        /// Files or directories (defaults to the root).
        paths: Vec<PathBuf>,
    },
    /// Install a git pre-commit hook that runs rabot on the files being committed.
    Hook {
        /// Replace an existing pre-commit hook.
        #[arg(long)]
        force: bool,
        /// Print the hook script instead of installing it.
        #[arg(long)]
        print: bool,
    },
    /// Write a rabot.toml with every rule at its default level.
    Init,
    /// List every rule with its default level and the article behind it.
    Rules,
}

/// Everything that can stop a command before it produces a report.
#[derive(Debug, thiserror::Error)]
enum CliError {
    #[error(transparent)]
    App(#[from] AppError),
    #[error(transparent)]
    Hook(#[from] HookError),
    #[error("cannot write output: {0}")]
    Output(#[from] std::io::Error),
}

impl Cli {
    /// Parse `args` (including the program name) and run to an exit code.
    pub fn main(args: impl IntoIterator<Item = OsString>) -> ExitCode {
        match Cli::parse_from(args).run() {
            Ok(code) => code,
            // The reader closed the pipe (`rabot check | head`): not our problem.
            Err(CliError::Output(error)) if error.kind() == std::io::ErrorKind::BrokenPipe => {
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("error: {error}");
                let mut source = std::error::Error::source(&error);
                while let Some(cause) = source {
                    eprintln!("  caused by: {cause}");
                    source = cause.source();
                }
                ExitCode::from(2)
            }
        }
    }

    fn run(self) -> Result<ExitCode, CliError> {
        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        let command = self.command.unwrap_or(Command::Check {
            changed: None,
            format: Format::Text,
            paths: Vec::new(),
            strict: false,
        });
        match command {
            Command::Check {
                changed,
                format,
                paths,
                strict,
            } => {
                let app = App::load(&self.root)?;
                let outcome = app.check(&Scope::from_flags(changed, paths))?;
                Report::new(format, &self.root).write(&outcome, &mut out)?;
                Ok(exit_code(&outcome, strict))
            }
            Command::Fmt {
                changed,
                check,
                diff,
                format,
                no_rustfmt,
                paths,
            } => {
                let app = App::load(&self.root)?;
                let check = check || diff;
                let mode = if check {
                    FormatMode::Check
                } else {
                    FormatMode::Write
                };
                let outcome = app.format(&Scope::from_flags(changed, paths), mode)?;
                if mode == FormatMode::Write && !no_rustfmt {
                    reindent(&outcome);
                }
                Report::new(format, &self.root)
                    .with_diff(diff)
                    .write(&outcome, &mut out)?;
                let would_change = check && !outcome.changed.is_empty();
                Ok(if outcome.has_errors() || would_change {
                    ExitCode::from(1)
                } else {
                    ExitCode::SUCCESS
                })
            }
            Command::Hook { force, print } => {
                if print {
                    write!(out, "{}", PreCommitHook::script())?;
                    return Ok(ExitCode::SUCCESS);
                }
                let hook = PreCommitHook::locate(&self.root)?;
                hook.install(force)?;
                writeln!(
                    out,
                    "installed {}: every commit now runs `rabot fmt --check --changed` and `rabot check --changed`",
                    hook.path.display()
                )?;
                Ok(ExitCode::SUCCESS)
            }
            Command::Init => {
                let path = self.root.join(FILE_NAME);
                if path.exists() {
                    writeln!(out, "{} already exists; leaving it alone", path.display())?;
                    return Ok(ExitCode::from(1));
                }
                std::fs::write(&path, Config::template())?;
                writeln!(out, "wrote {}", path.display())?;
                Ok(ExitCode::SUCCESS)
            }
            Command::Rules => {
                for rule in Rule::all() {
                    let fix = if rule.fixable() {
                        " (fixed by `rabot fmt`)"
                    } else {
                        ""
                    };
                    writeln!(
                        out,
                        "{:<24} {:<8} {}{fix}",
                        rule.name(),
                        rule.default_level(),
                        rule.description()
                    )?;
                    writeln!(out, "{:<24} {:<8} {}", "", "", rule.reference())?;
                }
                Ok(ExitCode::SUCCESS)
            }
        }
    }
}

/// `ExitCode` is not ours to `impl` on, and `Outcome` does not know about
/// process exit codes, so this stays a free function.
// rabot: allow(free-function) ExitCode is a foreign type; Outcome is process-agnostic
fn exit_code(outcome: &Outcome, strict: bool) -> ExitCode {
    if outcome.has_errors() || (strict && outcome.count(Level::Warn) > 0) {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

/// rustfmt every file rabot rewrote. A missing rustfmt is a note, not an
/// error: the reordering already happened and is valid Rust.
// rabot: allow(free-function) Outcome is a report; running external tools on it is the CLI's job
fn reindent(outcome: &Outcome) {
    if outcome.changed.is_empty() {
        return;
    }
    if !Rustfmt::available() {
        eprintln!("note: rustfmt not found; run `cargo fmt` to re-indent the reordered files");
        return;
    }
    for change in &outcome.changed {
        if let Err(error) = Rustfmt::format(&change.path) {
            eprintln!("warning: {error}");
        }
    }
}
