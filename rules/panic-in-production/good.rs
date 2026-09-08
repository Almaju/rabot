fn get_user(id: &UserId, db: &Database) -> Result<User, NotFoundError> {
    db.query("SELECT * FROM users WHERE id = $1", id)
        .ok_or(NotFoundError { user_id: id.clone() })
}

// Startup: the program cannot run without these. Panicking is honest here.
fn main() {
    let config = load_config().expect("config file required for startup");
    serve(config);
}
