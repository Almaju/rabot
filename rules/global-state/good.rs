struct Users {
    db: Database,
}

impl Users {
    async fn load(&self, id: &UserId) -> Result<User, LoadError> {
        self.db.query(id).await
    }
}

#[tokio::main]
async fn main() -> Result<(), StartupError> {
    // every dependency constructed in one place, then handed down
    let config = Config::from_env()?;
    let db = Database::connect(&config.db_url).await?;
    let users = Users { db };
    Api::new(users).serve().await
}
