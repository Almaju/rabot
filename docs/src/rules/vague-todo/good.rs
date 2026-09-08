impl Users {
    // TODO: this linear scan works at current scale (~500 users) but will
    // need an index once we hit the enterprise tier. See PERF-112.
    fn find_by_email(&self, email: &Email) -> Option<&User> {
        self.users.iter().find(|user| user.email == *email)
    }
}
