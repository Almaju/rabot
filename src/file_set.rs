use std::path::{Path, PathBuf};

use ignore::WalkBuilder;
use ignore::overrides::OverrideBuilder;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FileSetError {
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

impl FileSet {
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
