# ambient-randomness

**Level**: warn · **Article**: [Tests](https://almaju.github.io/blog/docs/fundamentals/architecture/testing)

## What it checks

`rand::random()`, `rand::thread_rng()`, `rand::rng()`, `StdRng::from_entropy()`
and friends, anywhere but `main`. A global generator makes the code
correct on average and impossible to replay when it is not.

## Don't

```rust
fn pick_winner(entries: &[Entry]) -> &Entry {
    &entries[rand::thread_rng().gen_range(0..entries.len())]
}
```

The bug report says "the same person won twice". You cannot reproduce it.

## Do

```rust
fn pick_winner<'a>(entries: &'a [Entry], rng: &mut impl Rng) -> &'a Entry {
    &entries[rng.gen_range(0..entries.len())]
}

// main: one generator, seeded once
let mut rng = StdRng::from_entropy();

// test: the same draw every time
let mut rng = StdRng::seed_from_u64(42);
```

## Silence it

```rust
// rabot: allow(ambient-randomness) jitter on a retry delay; the exact value never matters
let jitter = rand::random::<u64>() % 50;
```
