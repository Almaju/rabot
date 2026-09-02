use std::path::{Path, PathBuf};

use ignore::WalkBuilder;
use ignore::overrides::OverrideBuilder;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FileSetError {
    #[error("git failed in {root}: {message}")]
    Git { message: String, root: PathBuf },
    #[error("cannot run git in {root}: {source}")]
    GitUnavailable {
        root: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("invalid exclude pattern `{pattern}`: {source}")]
    Pattern {
        pattern: String,
        #[source]
        source: ignore::Error,
    },
    #[error("cannot walk {path}: {source}")]
    Walk {
        path: PathBuf,
        #[source]
        source: ignore::Error,
    },
}

/// The `.rs` files a run looks at: explicit files as given, directories
/// walked with `.gitignore` respected and the configured excludes applied.
#[derive(Debug, Default)]
pub struct FileSet {
    paths: Vec<PathBuf>,
}

/// Which files a run looks at.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Scope {
    /// Files git considers changed: modified, added or untracked relative to
    /// `since` (a ref), or relative to `HEAD` when `since` is `None`.
    Changed { since: Option<String> },
    /// Explicit files and directories.
    Paths(Vec<PathBuf>),
}

impl Scope {
    /// `--changed` wins over explicit paths; `--changed` alone means "since
    /// HEAD", which is what git itself means by uncommitted changes.
    pub fn from_flags(changed: Option<String>, paths: Vec<PathBuf>) -> Self {
        match changed {
            Some(since) if since == "HEAD" => Scope::Changed { since: None },
            Some(since) => Scope::Changed { since: Some(since) },
            None => Scope::Paths(paths),
        }
    }
}

impl FileSet {
    /// The `.rs` files git reports as added, copied, modified or renamed
    /// since `since` (default `HEAD`), plus untracked ones. This is how a
    /// codebase that predates rabot migrates: on contact, file by file.
    pub fn changed(root: &Path, since: Option<&str>, excludes: &[String]) -> Result<Self, FileSetError> {
        let mut args = vec!["diff", "--name-only", "--diff-filter=ACMR", "--relative"];
        match since {
            Some(since) => args.push(since),
            // A repository with no commit yet has no HEAD to diff against;
            // everything staged is "changed".
            None if has_head(root) => args.push("HEAD"),
            None => args.push("--cached"),
        }
        let mut names = git(root, &args)?;
        names.extend(git(root, &["ls-files", "--others", "--exclude-standard"])?);
        let mut overrides = OverrideBuilder::new(root);
        for pattern in excludes {
            overrides
                .add(&format!("!{pattern}"))
                .map_err(|source| FileSetError::Pattern {
                    pattern: pattern.clone(),
                    source,
                })?;
        }
        let overrides = overrides.build().map_err(|source| FileSetError::Pattern {
            pattern: excludes.join(", "),
            source,
        })?;
        let mut paths: Vec<PathBuf> = names
            .into_iter()
            .filter(|name| name.ends_with(".rs"))
            .map(|name| root.join(name))
            .filter(|path| path.is_file() && !excluded(&overrides, root, path))
            .collect();
        paths.sort();
        paths.dedup();
        Ok(Self { paths })
    }

    pub fn discover(roots: &[PathBuf], excludes: &[String]) -> Result<Self, FileSetError> {
        let mut paths = Vec::new();
        for root in roots {
            if root.is_file() {
                paths.push(root.clone());
                continue;
            }
            let mut overrides = OverrideBuilder::new(root);
            for pattern in excludes {
                overrides
                    .add(&format!("!{pattern}"))
                    .map_err(|source| FileSetError::Pattern {
                        pattern: pattern.clone(),
                        source,
                    })?;
            }
            let overrides = overrides.build().map_err(|source| FileSetError::Pattern {
                pattern: excludes.join(", "),
                source,
            })?;
            let walker = WalkBuilder::new(root).overrides(overrides).hidden(true).build();
            for entry in walker {
                let entry = entry.map_err(|source| FileSetError::Walk {
                    path: root.clone(),
                    source,
                })?;
                let path = entry.path();
                if path.is_file() && path.extension().is_some_and(|ext| ext == "rs") {
                    paths.push(path.to_path_buf());
                }
            }
        }
        paths.sort();
        paths.dedup();
        Ok(Self { paths })
    }

    pub fn is_empty(&self) -> bool {
        self.paths.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Path> {
        self.paths.iter().map(PathBuf::as_path)
    }

    pub fn len(&self) -> usize {
        self.paths.len()
    }
}

/// Whether `path` or any directory between `root` and it matches an exclude
/// pattern. gitignore patterns name a directory, not its descendants, so each
/// ancestor is checked as a directory.
fn excluded(overrides: &ignore::overrides::Override, root: &Path, path: &Path) -> bool {
    let Ok(relative) = path.strip_prefix(root) else {
        return false;
    };
    let mut ancestor = root.to_path_buf();
    for component in relative.components() {
        ancestor.push(component);
        let is_dir = ancestor != path;
        if overrides.matched(&ancestor, is_dir).is_ignore() {
            return true;
        }
    }
    false
}

fn has_head(root: &Path) -> bool {
    git(root, &["rev-parse", "--verify", "--quiet", "HEAD"]).is_ok()
}

fn git(root: &Path, args: &[&str]) -> Result<Vec<String>, FileSetError> {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .map_err(|source| FileSetError::GitUnavailable {
            root: root.to_path_buf(),
            source,
        })?;
    if !output.status.success() {
        return Err(FileSetError::Git {
            message: String::from_utf8_lossy(&output.stderr).trim().to_string(),
            root: root.to_path_buf(),
        });
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::to_string)
        .filter(|line| !line.is_empty())
        .collect())
}
