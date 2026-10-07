impl User {
    fn from_signup(input: Signup, now: DateTime) -> Self {
        User {
            role: Role::Member,
            name: input.name,
            id: UserId::new(),
            email: input.email,
            created_at: now,
        }
    }
}
