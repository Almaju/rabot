impl Users {
    fn find_by_email(&self, email: &Email) -> Option<&User> {
        // TODO: refactor this
        // FIXME
        self.users.iter().find(|user| user.email == *email)
    }
}
