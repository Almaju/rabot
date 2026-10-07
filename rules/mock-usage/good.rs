struct MemDatabase {
    users: Mutex<HashMap<UserId, User>>,
}

impl Database for MemDatabase {
    fn find_by_email(&self, email: &Email) -> Option<User> {
        let users = self.users.lock().unwrap_or_else(PoisonError::into_inner);
        users.values().find(|user| user.email == *email).cloned()
    }

    fn insert(&self, user: NewUser) -> Result<User, DbError> {
        let mut users = self.users.lock().unwrap_or_else(PoisonError::into_inner);
        if users.values().any(|existing| existing.email == user.email) {
            return Err(DbError::UniqueViolation("email"));
        }
        let user = User::from(user);
        users.insert(user.id.clone(), user.clone());
        Ok(user)
    }
}

#[test]
fn rejects_a_duplicate_email() {
    let db = MemDatabase::default();
    db.insert(NewUser::named("ada", "ada@example.com")).unwrap();
    let duplicate = db.insert(NewUser::named("ada again", "ada@example.com"));
    assert!(matches!(duplicate, Err(DbError::UniqueViolation("email"))));
}
