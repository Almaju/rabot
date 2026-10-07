fn validate_email(s: &str) -> bool {
    s.contains('@') && !s.starts_with('@')
}

fn register(input: &str) -> Result<(), ApiError> {
    if !validate_email(input) {
        return Err(ApiError::Invalid("email")); // which rule? the user has to guess
    }
    Ok(())
}
