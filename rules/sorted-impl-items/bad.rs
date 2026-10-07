impl Users {
    fn exists(&self, id: &UserId) -> bool {
        self.db.contains(id)
    }

    pub fn delete(&self, id: &UserId) {
        self.db.remove(id);
    }

    pub fn create(&self, user: User) {
        self.db.insert(user);
    }

    pub fn new(db: Database) -> Self {
        Self { db }
    }
}
