//! Cargo packages under the root, so that `-p name` can narrow a run to one
//! crate of a workspace the way `cargo -p` does.

use std::path::{Path, PathBuf};

use ignore::WalkBuilder;
use serde::Deserialize;
use thiserror::Error;

const MANIFEST: &str = "Cargo.toml";

#[derive(Debug, Error)]
pub enum PackageError {
    #[error("cannot parse {path}: {source}")]
    Manifest {
        path: PathBuf,
        #[source]
        source: Box<toml::de::Error>,
    },
    #[error("cannot read {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("no package named `{name}` under {root}; found: {available}")]
    Unknown {
        available: String,
        name: String,
        root: PathBuf,
    },
    #[error("cannot walk {path}: {source}")]
    Walk {
        path: PathBuf,
        #[source]
        source: ignore::Error,
    },
}

/// One crate: its name from `[package]` and the directory of its manifest.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Package {
    /// `dir` resolved once, for [`Packages::owner`].
    canonical_dir: PathBuf,
    pub dir: PathBuf,
    pub name: String,
}

/// Every package whose `Cargo.toml` sits under the root. Workspace manifests
/// without a `[package]` section contribute nothing; their members are found
/// by walking, so globbed `members` need no special handling.
#[derive(Clone, Debug, Default)]
pub struct Packages {
    items: Vec<Package>,
}

/// The part of a manifest rabot reads.
#[derive(Deserialize)]
struct Manifest {
    package: Option<ManifestPackage>,
}

#[derive(Deserialize)]
struct ManifestPackage {
    name: String,
}

impl Packages {
    /// Walk `root` for manifests, respecting `.gitignore` and skipping
    /// `target` directories.
    pub fn discover(root: &Path) -> Result<Self, PackageError> {
        let walker = WalkBuilder::new(root)
            .hidden(true)
            .filter_entry(|entry| entry.file_name() != "target")
            .build();
        let mut items = Vec::new();
        for entry in walker {
            let entry = entry.map_err(|source| PackageError::Walk {
                path: root.to_path_buf(),
                source,
            })?;
            let path = entry.path();
            if path.file_name().is_none_or(|name| name != MANIFEST) || !path.is_file() {
                continue;
            }
            let text = std::fs::read_to_string(path).map_err(|source| PackageError::Read {
                path: path.to_path_buf(),
                source,
            })?;
            let manifest: Manifest = toml::from_str(&text).map_err(|source| PackageError::Manifest {
                path: path.to_path_buf(),
                source: Box::new(source),
            })?;
            if let (Some(package), Some(dir)) = (manifest.package, path.parent()) {
                items.push(Package {
                    canonical_dir: comparable(dir),
                    dir: dir.to_path_buf(),
                    name: package.name,
                });
            }
        }
        items.sort_by(|a, b| a.dir.cmp(&b.dir));
        Ok(Self { items })
    }

    /// The package a file belongs to: the one whose directory is its deepest
    /// ancestor. A workspace root that is itself a package does not own the
    /// files of the members nested inside it.
    pub fn owner(&self, path: &Path) -> Option<&Package> {
        let path = comparable(path);
        self.items
            .iter()
            .filter(|package| path.starts_with(&package.canonical_dir))
            .max_by_key(|package| package.canonical_dir.components().count())
    }

    /// The packages called `names`, or an error naming the first unknown one
    /// alongside every name that does exist.
    pub fn select(&self, root: &Path, names: &[String]) -> Result<Vec<Package>, PackageError> {
        names
            .iter()
            .map(|name| {
                self.items
                    .iter()
                    .find(|package| &package.name == name)
                    .cloned()
                    .ok_or_else(|| PackageError::Unknown {
                        available: self.names().join(", "),
                        name: name.clone(),
                        root: root.to_path_buf(),
                    })
            })
            .collect()
    }

    fn names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.items.iter().map(|package| package.name.as_str()).collect();
        names.sort_unstable();
        names
    }
}

/// `path` in a form two spellings of the same file agree on: absolute and
/// with symlinks resolved when it exists, as given otherwise.
fn comparable(path: &Path) -> PathBuf {
    // rabot: allow(dropped-error-context) a path that cannot be resolved is compared as given, not reported
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn workspace() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("rabot-packages-{}", std::process::id()));
        let write = |relative: &str, text: &str| {
            let path = dir.join(relative);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        };
        write(
            "Cargo.toml",
            "[package]\nname = \"app\"\n[workspace]\nmembers = [\"crates/*\"]\n",
        );
        write("crates/core/Cargo.toml", "[package]\nname = \"core\"\n");
        write("crates/virtual/Cargo.toml", "[workspace]\n");
        dir
    }

    #[test]
    fn finds_packages_and_their_owners() {
        let root = workspace();
        let packages = Packages::discover(&root).unwrap();
        assert_eq!(packages.names(), vec!["app", "core"]);
        let owner = |relative: &str| packages.owner(&root.join(relative)).map(|p| p.name.as_str());
        assert_eq!(owner("src/main.rs"), Some("app"));
        assert_eq!(owner("crates/core/src/lib.rs"), Some("core"));
        let error = packages.select(&root, &["nope".to_string()]).unwrap_err();
        assert!(error.to_string().contains("found: app, core"), "{error}");
        std::fs::remove_dir_all(&root).ok();
    }
}
