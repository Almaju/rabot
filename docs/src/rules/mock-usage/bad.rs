use mockall::automock;

#[automock]
trait Database {
    fn find_by_email(&self, email: &Email) -> Option<User>;
}

#[test]
fn registers_a_new_user() {
    let mut db = MockDatabase::new();
    db.expect_find_by_email().return_const(None); // "no duplicate, go ahead"
    assert!(Users::new(db).register(Email::parse("ada@example.com")).is_ok());
}
