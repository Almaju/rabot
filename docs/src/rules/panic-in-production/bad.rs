fn get_user(id: &UserId, db: &Database) -> User {
    db.query("SELECT * FROM users WHERE id = $1", id).unwrap()
}
