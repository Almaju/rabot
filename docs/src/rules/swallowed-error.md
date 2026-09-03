# swallowed-error

**Level**: warn · **Article**: [Errors](https://almaju.github.io/blog/docs/fundamentals/modeling/errors)

> Every silent catch is a future 3am.

## What it checks

Three shapes that make a failure disappear:

- an empty `Err` arm: `Err(_) => {}`
- an empty `if let Err(..) = .. {}`
- `.ok();` as a statement, which converts the `Result` to an `Option` and
  throws it away

## Don't

```rust
match cache.invalidate(&key) {
    Ok(()) => {}
    Err(_) => {} // shouldn't happen
}

std::fs::remove_file(&tmp).ok();
```

It happened. The comment lied. Somebody will spend four hours finding which
branch swallowed it.

## Do

```rust
if let Err(error) = cache.invalidate(&key) {
    warn!(%key, %error, "cache entry survives invalidation; serving stale until TTL");
}

std::fs::remove_file(&tmp)?;
```

Propagate it, or log it with the context the reader at 3am needs. Either
way, the failure leaves a trace.

## Silence it

```rust
// rabot: allow(swallowed-error) best-effort cleanup of a temp file; the OS reclaims it anyway
std::fs::remove_file(&tmp).ok();
```
