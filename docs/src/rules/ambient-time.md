# ambient-time

**Level**: warn · **Article**: [Tests](https://almaju.github.io/blog/docs/fundamentals/architecture/testing)

> Some things are genuinely hard to test: the clock, random number
> generators, external HTTP calls. The instinct is to mock them. The right
> move is to make them injectable.

## What it checks

A call to `SystemTime::now()`, `Instant::now()`, `Utc::now()`,
`Local::now()`, `OffsetDateTime::now_utc()` or the like anywhere but `main`.
The wall clock is a dependency, and this one is hidden from every signature
that uses it.

## Don't

```rust
{{#include ambient-time/bad.rs}}
```

The test for "expires one hour after creation" has to compute the current
hour, or sleep, or give up.

## Do

```rust
{{#include ambient-time/good.rs}}
```

The bar for injection: would it be useful outside tests? A `Clock` is. You
freeze time in staging, simulate midnight rollover in a demo, replay a
historical scenario.

## Silence it

```rust
let started = Instant::now(); // rabot: allow(ambient-time) request timing for the log line; nothing branches on it
```
