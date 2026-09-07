struct User {
    email: Email,
    id: UserId,
    latitude: Latitude,
}

struct Email(String);

impl Email {
    fn parse(s: &str) -> Result<Self, ValidationError> {
        if !s.contains('@') {
            return Err(ValidationError::MissingAt);
        }
        Ok(Self(s.to_lowercase()))
    }
}
