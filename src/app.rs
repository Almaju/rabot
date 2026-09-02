use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::config::{Config, ConfigError};
use crate::diagnostic::{Diagnostic, Level, Position};
use crate::edit::{EditError, Edits};
use crate::file_set::{FileSet, FileSetError};
use crate::rule::Rule;
use crate::rules::{Context, Findings, LocalTypes};
use crate::source_file::SourceFile;

/// Formatting passes before rabot gives up on a file that keeps changing.
const MAX_PASSES: usize = 8;

#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    Config(#[from] ConfigError),
    #[error("{path}: {source}")]
    Edit {
        path: PathBuf,
        #[source]
        source: EditError,
    },
    #[error(transparent)]
    Files(#[from] FileSetError),
    #[error("cannot read {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("cannot write {path}: {source}")]
    Write {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

/// What `rabot fmt` does with the files it would change.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FormatMode {
    /// Report the files that would change; touch nothing.
    Check,
    /// Rewrite the files in place.
    Write,
}

/// The result of one run, whichever command produced it.
#[derive(Debug, Default)]
pub struct Outcome {
    pub changed: Vec<PathBuf>,
    pub diagnostics: Vec<Diagnostic>,
    pub files_seen: usize,
}

impl Outcome {
    pub fn count(&self, level: Level) -> usize {
        self.diagnostics.iter().filter(|d| d.level == level).count()
    }

    pub fn has_errors(&self) -> bool {
        self.count(Level::Error) > 0
    }

    fn finish(mut self) -> Self {
        self.diagnostics.sort_by_key(Diagnostic::sort_key);
        self.diagnostics.dedup();
        self
    }
}

/// rabot itself: a configuration and the root it applies to.
pub struct App {
    pub config: Config,
    pub root: PathBuf,
}

impl App {
    pub fn load(root: &Path) -> Result<Self, AppError> {
        Ok(Self::new(Config::load(root)?, root.to_path_buf()))
    }

    pub fn new(config: Config, root: PathBuf) -> Self {
        Self { config, root }
    }

    /// Lint: every rule, every diagnostic, nothing written.
    pub fn check(&self, roots: &[PathBuf]) -> Result<Outcome, AppError> {
        let mut outcome = Outcome::default();
        let files = self.parse_all(roots, &mut outcome)?;
        let local_types = LocalTypes::collect(&files);
        for file in &files {
            let findings = self.findings(file, &local_types);
            outcome.diagnostics.extend(findings.diagnostics);
        }
        Ok(outcome.finish())
    }

    /// Format: apply every fix the sorting rules offer, repeating until the
    /// file is stable, then write it back (or just report, in check mode).
    pub fn format(&self, roots: &[PathBuf], mode: FormatMode) -> Result<Outcome, AppError> {
        let mut outcome = Outcome::default();
        let files = self.parse_all(roots, &mut outcome)?;
        let local_types = LocalTypes::collect(&files);
        for file in files {
            let path = file.path.clone();
            let mut current = file;
            let mut first_diagnostics = None;
            for _ in 0..MAX_PASSES {
                let findings = self.findings(&current, &local_types);
                let sorting: Vec<Diagnostic> = findings
                    .diagnostics
                    .into_iter()
                    .filter(|diagnostic| diagnostic.rule.fixable())
                    .collect();
                if first_diagnostics.is_none() {
                    first_diagnostics = Some(sorting);
                }
                if findings.edits.is_empty() {
                    break;
                }
                let edits = Edits::new(findings.edits);
                let text = edits.apply(&current.text).map_err(|source| AppError::Edit {
                    path: path.clone(),
                    source,
                })?;
                current = match SourceFile::parse(path.clone(), text) {
                    Ok(reparsed) => reparsed,
                    Err(error) => {
                        outcome.diagnostics.push(Diagnostic {
                            help: Some("this is a rabot bug; the file was left untouched".to_string()),
                            level: Level::Error,
                            message: format!("formatting produced invalid Rust: {}", error.message),
                            path: path.clone(),
                            position: Position {
                                column: error.column,
                                line: error.line,
                            },
                            rule: Rule::SyntaxError,
                        });
                        current = SourceFile::parse(path.clone(), self.read(&path)?).map_err(|_| {
                            AppError::Read {
                                path: path.clone(),
                                source: std::io::Error::other("file changed while formatting"),
                            }
                        })?;
                        break;
                    }
                };
            }
            let original = self.read(&path)?;
            if current.text != original {
                outcome.changed.push(path.clone());
                match mode {
                    FormatMode::Write => {
                        std::fs::write(&path, &current.text).map_err(|source| AppError::Write {
                            path: path.clone(),
                            source,
                        })?;
                    }
                    FormatMode::Check => {
                        outcome.diagnostics.extend(first_diagnostics.unwrap_or_default());
                    }
                }
            }
        }
        Ok(outcome.finish())
    }

    fn findings(&self, file: &SourceFile, local_types: &LocalTypes) -> Findings {
        let cx = Context {
            config: &self.config,
            file,
            local_types,
        };
        cx.run_all()
    }

    fn parse_all(&self, roots: &[PathBuf], outcome: &mut Outcome) -> Result<Vec<SourceFile>, AppError> {
        let roots: Vec<PathBuf> = if roots.is_empty() {
            vec![self.root.clone()]
        } else {
            roots.to_vec()
        };
        let set = FileSet::discover(&roots, &self.config.files.exclude)?;
        outcome.files_seen = set.len();
        let mut files = Vec::with_capacity(set.len());
        for path in set.iter() {
            let text = self.read(path)?;
            match SourceFile::parse(path, text) {
                Ok(file) => files.push(file),
                Err(error) => outcome.diagnostics.push(Diagnostic {
                    help: None,
                    level: Level::Error,
                    message: error.message,
                    path: error.path,
                    position: Position {
                        column: error.column,
                        line: error.line,
                    },
                    rule: Rule::SyntaxError,
                }),
            }
        }
        Ok(files)
    }

    fn read(&self, path: &Path) -> Result<String, AppError> {
        std::fs::read_to_string(path).map_err(|source| AppError::Read {
            path: path.to_path_buf(),
            source,
        })
    }
}
