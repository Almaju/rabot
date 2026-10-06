use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::config::{Config, ConfigError};
use crate::diagnostic::{Diagnostic, Level, Position};
use crate::edit::{EditError, Edits};
use crate::file_set::{FileSet, FileSetError, Scope};
use crate::module_graph::ModuleGraph;
use crate::package::{PackageError, Packages};
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
    #[error(transparent)]
    Package(#[from] PackageError),
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormatMode {
    /// Report the files that would change; touch nothing.
    Check,
    /// Rewrite the files in place.
    Write,
}

/// A file `fmt` rewrote, or would rewrite in check mode.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Change {
    pub after: String,
    pub before: String,
    pub path: PathBuf,
}

/// The result of one run, whichever command produced it.
#[derive(Debug, Default)]
pub struct Outcome {
    pub changed: Vec<Change>,
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

/// rabot itself: a configuration, the root it applies to, and the packages
/// a run is narrowed to (every file under the root when there are none).
pub struct App {
    pub config: Config,
    pub packages: Vec<String>,
    pub root: PathBuf,
}

impl App {
    pub fn load(root: &Path) -> Result<Self, AppError> {
        Ok(Self::new(Config::load(root)?, root.to_path_buf()))
    }

    pub fn new(config: Config, root: PathBuf) -> Self {
        Self {
            config,
            packages: Vec::new(),
            root,
        }
    }

    /// Lint: every rule, every diagnostic, nothing written.
    pub fn check(&self, scope: &Scope) -> Result<Outcome, AppError> {
        let mut outcome = Outcome::default();
        let files = self.parse_all(scope, &mut outcome)?;
        let local_types = LocalTypes::collect(&files);
        let module_graph = self.module_graph(&files);
        for file in &files {
            let findings = self.findings(file, &local_types, &module_graph);
            outcome.diagnostics.extend(findings.diagnostics);
        }
        Ok(outcome.finish())
    }

    /// Format: apply every fix the sorting rules offer, repeating until the
    /// file is stable, then write it back (or just report, in check mode).
    pub fn format(&self, scope: &Scope, mode: FormatMode) -> Result<Outcome, AppError> {
        let mut outcome = Outcome::default();
        let files = self.parse_all(scope, &mut outcome)?;
        let local_types = LocalTypes::collect(&files);
        let module_graph = self.module_graph(&files);
        for file in files {
            let path = file.path.clone();
            let mut current = file;
            let mut first_diagnostics = None;
            for _ in 0..MAX_PASSES {
                let findings = self.findings(&current, &local_types, &module_graph);
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
                        current = SourceFile::parse(path.clone(), self.read(&path)?).map_err(|error| {
                            AppError::Read {
                                path: path.clone(),
                                source: std::io::Error::other(format!(
                                    "file changed while formatting: {error}"
                                )),
                            }
                        })?;
                        break;
                    }
                };
            }
            let original = self.read(&path)?;
            if current.text != original {
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
                outcome.changed.push(Change {
                    after: current.text,
                    before: original,
                    path,
                });
            }
        }
        Ok(outcome.finish())
    }

    /// Narrow every run to the files of these Cargo packages, as `-p` does.
    pub fn with_packages(mut self, packages: Vec<String>) -> Self {
        self.packages = packages;
        self
    }

    /// The files in `scope`, narrowed to the selected packages. A file
    /// belongs to the package whose manifest is its nearest ancestor, so
    /// selecting a workspace root package leaves its members out.
    fn file_set(&self, scope: &Scope) -> Result<FileSet, AppError> {
        let excludes = &self.config.files.exclude;
        let packages = if self.packages.is_empty() {
            None
        } else {
            let all = Packages::discover(&self.root)?;
            let selected = all.select(&self.root, &self.packages)?;
            Some((all, selected))
        };
        let set = match (scope, &packages) {
            (Scope::Changed { since }, _) => FileSet::changed(&self.root, since.as_deref(), excludes)?,
            (Scope::Paths(roots), Some((_, selected))) if roots.is_empty() => {
                let dirs: Vec<PathBuf> = selected.iter().map(|package| package.dir.clone()).collect();
                FileSet::discover(&dirs, excludes)?
            }
            (Scope::Paths(roots), _) if roots.is_empty() => {
                FileSet::discover(std::slice::from_ref(&self.root), excludes)?
            }
            (Scope::Paths(roots), _) => FileSet::discover(roots, excludes)?,
        };
        Ok(match packages {
            Some((all, selected)) => set.retain(|path| {
                all.owner(path)
                    .is_some_and(|owner| selected.iter().any(|package| package.name == owner.name))
            }),
            None => set,
        })
    }

    fn findings(&self, file: &SourceFile, local_types: &LocalTypes, module_graph: &ModuleGraph) -> Findings {
        let cx = Context {
            config: &self.config,
            file,
            local_types,
            module_graph,
        };
        cx.run_all()
    }

    /// The module graph costs a read of every crate in scope; skip it when
    /// nothing would report from it.
    fn module_graph(&self, files: &[SourceFile]) -> ModuleGraph {
        if self.config.level(Rule::ModuleCycle) == Level::Allow {
            ModuleGraph::default()
        } else {
            ModuleGraph::build(files)
        }
    }

    fn parse_all(&self, scope: &Scope, outcome: &mut Outcome) -> Result<Vec<SourceFile>, AppError> {
        let set = self.file_set(scope)?;
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
