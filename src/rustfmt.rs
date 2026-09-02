use std::path::{Path, PathBuf};
use std::process::Command;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum RustfmtError {
    #[error("rustfmt failed on {path}: {message}")]
    Failed { message: String, path: PathBuf },
    #[error("cannot run rustfmt: {0}")]
    Unavailable(#[source] std::io::Error),
}

/// rabot reorders; rustfmt re-indents. After `rabot fmt` rewrites a file,
/// rustfmt runs on it with the edition of the nearest `Cargo.toml`, so the
/// result is what `cargo fmt` would have produced.
#[derive(Debug)]
pub struct Rustfmt;

impl Rustfmt {
    /// True when a `rustfmt` binary can be found on `PATH`.
    pub fn available() -> bool {
        Command::new("rustfmt")
            .arg("--version")
            .output()
            .is_ok_and(|output| output.status.success())
    }

    pub fn format(path: &Path) -> Result<(), RustfmtError> {
        let output = Command::new("rustfmt")
            .arg("--edition")
            .arg(edition_of(path))
            .arg(path)
            .output()
            .map_err(RustfmtError::Unavailable)?;
        if output.status.success() {
            return Ok(());
        }
        Err(RustfmtError::Failed {
            message: String::from_utf8_lossy(&output.stderr).trim().to_string(),
            path: path.to_path_buf(),
        })
    }
}

/// The `edition` of the nearest `Cargo.toml` above `path`, or 2021.
fn edition_of(path: &Path) -> String {
    path.ancestors()
        .skip(1)
        .map(|dir| dir.join("Cargo.toml"))
        .find(|manifest| manifest.is_file())
        .and_then(|manifest| std::fs::read_to_string(manifest).ok())
        .and_then(|text| text.parse::<toml::Table>().ok())
        .and_then(|table| {
            let package = table.get("package")?.as_table()?;
            match package.get("edition")? {
                toml::Value::String(edition) => Some(edition.clone()),
                // `edition.workspace = true`: look at the workspace table.
                _ => table
                    .get("workspace")?
                    .get("package")?
                    .get("edition")?
                    .as_str()
                    .map(str::to_string),
            }
        })
        .unwrap_or_else(|| "2021".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_edition_from_manifest() {
        let dir = std::env::temp_dir().join(format!("rabot-edition-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::write(
            dir.join("Cargo.toml"),
            "[package]\nname = \"x\"\nedition = \"2018\"\n",
        )
        .unwrap();
        assert_eq!(edition_of(&dir.join("src/lib.rs")), "2018");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn defaults_without_manifest() {
        assert_eq!(edition_of(Path::new("/definitely/not/here/x.rs")), "2021");
    }
}
