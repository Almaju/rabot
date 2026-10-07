trait Clock: Send + Sync {
    fn now(&self) -> DateTime<Utc>;
}

struct Sessions<C: Clock> {
    clock: C,
}

impl<C: Clock> Sessions<C> {
    fn create(&self, user_id: UserId) -> Session {
        let now = self.clock.now();
        Session {
            created_at: now,
            expires_at: now + Duration::hours(1),
            user_id,
        }
    }
}

#[test]
fn expires_one_hour_after_creation() {
    let clock = FixedClock(Utc.with_ymd_and_hms(2024, 1, 15, 12, 0, 0).unwrap());
    let session = Sessions { clock }.create(UserId::new());
    assert_eq!(session.expires_at.hour(), 13);
}
