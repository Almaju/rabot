use std::path::{Path, PathBuf};
use std::process::Command;

use thiserror::Error;

const SCRIPT: &str = "#!/bin/sh
# Installed by `rabot hook`. Sort what you touch; leave the rest alone.
# https://github.com/almaju/rabot
set -e
rabot fmt --check --changed >/dev/null 2>&1 || {
    echo 'rabot: some staged files need reordering. Run `rabot fmt --changed`, then stage the result.' >&2
    rabot fmt --diff --changed >&2
    exit 1
}
rabot check --changed
";

#[derive(Debug, Error)]
pub enum HookError {
    #[error("{path} already exists; pass --force to replace it")]
    Exists { path: PathBuf },
    #[error("cannot run git: {0}")]
    GitUnavailable(#[source] std::io::Error),
    #[error("{root} is not inside a git repository: {message}")]
    NotARepository { message: String, root: PathBuf },
    #[error("cannot write {path}: {source}")]
    Write {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

/// The git pre-commit hook that runs rabot on what is about to be committed.
#[derive(Debug)]
pub struct PreCommitHook {
    pub path: PathBuf,
}

impl PreCommitHook {
    /// Locate the hook file for the repository containing `root`.
    pub fn locate(root: &Path) -> Result<Self, HookError> {
        let output = Command::new("git")
            .arg("-C")
            .arg(root)
            .args(["rev-parse", "--git-path", "hooks"])
            .output()
            .map_err(HookError::GitUnavailable)?;
        if !output.status.success() {
            return Err(HookError::NotARepository {
                message: String::from_utf8_lossy(&output.stderr).trim().to_string(),
                root: root.to_path_buf(),
            });
        }
        let hooks = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let hooks = if Path::new(&hooks).is_absolute() {
            PathBuf::from(hooks)
        } else {
            root.join(hooks)
        };
        Ok(Self {
            path: hooks.join("pre-commit"),
        })
    }

    pub fn install(&self, force: bool) -> Result<(), HookError> {
        if self.path.exists() && !force {
            return Err(HookError::Exists {
                path: self.path.clone(),
            });
        }
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).map_err(|source| HookError::Write {
                path: self.path.clone(),
                source,
            })?;
        }
        std::fs::write(&self.path, SCRIPT).map_err(|source| HookError::Write {
            path: self.path.clone(),
            source,
        })?;
        make_executable(&self.path).map_err(|source| HookError::Write {
            path: self.path.clone(),
            source,
        })
    }

    pub fn script() -> &'static str {
        SCRIPT
    }
}

#[cfg(unix)]
fn make_executable(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut permissions = std::fs::metadata(path)?.permissions();
    permissions.set_mode(permissions.mode() | 0o111);
    std::fs::set_permissions(path, permissions)
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) -> std::io::Result<()> {
    Ok(())
}
