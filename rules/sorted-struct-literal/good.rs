impl User {
    fn from_signup(input: Signup, now: DateTime) -> Self {
        User {
            created_at: now,
            email: input.email,
            id: UserId::new(),
            name: input.name,
            role: Role::Member,
        }
    }
}
