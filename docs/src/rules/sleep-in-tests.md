# sleep-in-tests

**Level**: warn · **Article**: [Tests](https://almaju.github.io/blog/docs/fundamentals/architecture/testing)

> Developers who re-run flaky tests twice before investigating.

## What it checks

`thread::sleep`, `tokio::time::sleep` or `task::sleep` inside test code.
This rule fires only in tests; sleeping in production code is a different
question.

## Don't

```rust
#[tokio::test]
async fn delivers_the_event() {
    bus.publish(event);
    tokio::time::sleep(Duration::from_millis(50)).await; // "enough time"
    assert_eq!(subscriber.received(), vec![event]);
}
```

Fifty milliseconds is enough on your laptop. On a loaded CI runner it is
not, once a week, and someone adds a zero.

## Do

```rust
#[tokio::test]
async fn delivers_the_event() {
    bus.publish(event);
    let received = subscriber.next().await; // waits for the event, not for time
    assert_eq!(received, event);
}
```

When the code under test measures time, inject the [clock](ambient-time.md)
and advance it: `clock.advance(Duration::from_secs(3600))` is instant and
exact.

## Silence it

```rust
// rabot: allow(sleep-in-tests) exercises the real timeout path against the in-process server
```
