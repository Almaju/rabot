enum ConfigError {
    Unreadable {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

impl Config {
    fn load(path: &Path) -> Result<Self, ConfigError> {
        let text = std::fs::read_to_string(path).map_err(|source| ConfigError::Unreadable {
            path: path.to_path_buf(),
            source,
        })?;
        text.parse()
    }
}
