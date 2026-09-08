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
{{#include swallowed-error/bad.rs}}
```

It happened. The comment lied. Somebody will spend four hours finding which
branch swallowed it.

## Do

```rust
{{#include swallowed-error/good.rs}}
```

Propagate it, or log it with the context the reader at 3am needs. Either
way, the failure leaves a trace.

## Silence it

```rust
// rabot: allow(swallowed-error) best-effort cleanup of a temp file; the OS reclaims it anyway
std::fs::remove_file(&tmp).ok();
```
