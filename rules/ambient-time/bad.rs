impl Session {
    fn create(user_id: UserId) -> Session {
        let now = Utc::now();
        Session {
            created_at: now,
            expires_at: now + Duration::hours(1),
            user_id,
        }
    }
}
