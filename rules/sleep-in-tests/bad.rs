#[tokio::test]
async fn delivers_the_event() {
    let (bus, subscriber) = Bus::with_subscriber();
    bus.publish(Event::UserBanned);
    tokio::time::sleep(Duration::from_millis(50)).await; // "enough time"
    assert_eq!(subscriber.received(), vec![Event::UserBanned]);
}
