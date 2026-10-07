impl Config {
    fn load(path: &Path) -> Result<Self, ConfigError> {
        let text = std::fs::read_to_string(path).map_err(|_| ConfigError::Unreadable)?;
        text.parse()
    }
}
