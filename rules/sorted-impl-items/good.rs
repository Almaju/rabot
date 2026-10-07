impl Users {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub fn create(&self, user: User) {
        self.db.insert(user);
    }

    pub fn delete(&self, id: &UserId) {
        self.db.remove(id);
    }

    fn exists(&self, id: &UserId) -> bool {
        self.db.contains(id)
    }
}
