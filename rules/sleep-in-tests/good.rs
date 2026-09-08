#[tokio::test]
async fn delivers_the_event() {
    let (bus, mut subscriber) = Bus::with_subscriber();
    bus.publish(Event::UserBanned);
    let received = subscriber.next().await; // waits for the event, not for time
    assert_eq!(received, Some(Event::UserBanned));
}
