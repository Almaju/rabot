struct User {
    banned: bool,
    id: UserId,
}

struct Store {
    db: Database,
}

impl User {
    fn ban(self) -> Self {
        Self { banned: true, ..self }
    }

    async fn save(&self, store: &Store) -> Result<(), SaveError> {
        store.db.upsert(self).await
    }
}
