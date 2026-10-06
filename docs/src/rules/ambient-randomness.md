# ambient-randomness

**Level**: warn · **Article**: [Tests](https://almaju.github.io/blog/docs/fundamentals/architecture/testing)

## What it checks

`rand::random()`, `rand::thread_rng()`, `rand::rng()`, `StdRng::from_entropy()`
and friends, anywhere but `main`. A global generator makes the code
correct on average and impossible to replay when it is not.

## Don't

```rust
{{#include ambient-randomness/bad.rs}}
```

The bug report says "the same person won twice". You cannot reproduce it.

## Do

```rust
{{#include ambient-randomness/good.rs}}
```

## Silence it

```rust
// rabot: allow(ambient-randomness) jitter on a retry delay; the exact value never matters
let jitter = rand::random::<u64>() % 50;
```
