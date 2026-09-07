static DATABASE: OnceLock<Database> = OnceLock::new();

impl User {
    async fn load(id: &UserId) -> Result<User, LoadError> {
        let db = DATABASE.get().ok_or(LoadError::NotConnected)?; // hidden dependency
        db.query(id).await
    }
}
