enum EmailError {
    MissingAt,
    MissingLocalPart,
}

struct Email(String);

impl Email {
    fn parse(s: &str) -> Result<Self, EmailError> {
        if !s.contains('@') {
            return Err(EmailError::MissingAt);
        }
        if s.starts_with('@') {
            return Err(EmailError::MissingLocalPart);
        }
        Ok(Email(s.to_lowercase()))
    }
}
