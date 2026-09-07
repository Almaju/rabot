# untyped-error

**Level**: warn · **Article**: [Errors](https://almaju.github.io/blog/docs/fundamentals/modeling/errors)

> With exceptions, you catch `Error` and guess. With typed errors, you match
> the variant and act. The difference is whether your recovery logic is a
> strategy or a prayer.

## What it checks

A function returns `Box<dyn Error>`, `anyhow::Result`/`anyhow::Error`,
`eyre`, or `Result<T, String>` / `Result<T, &str>`. `fn main` and trait
impls are skipped: `main` may bubble anything, and `Error::source` is std's
signature, not yours.

## Don't

```rust
{{#include untyped-error/bad.rs}}
```

The caller can display the error. It cannot retry on a timeout, refresh on
an expired token, and give up on a validation failure, because it cannot
tell them apart.

## Do

```rust
{{#include untyped-error/good.rs}}
```

Granular enough to make different decisions. If two failures get identical
handling, they are one variant.

## Silence it

```rust
// rabot: allow(untyped-error) CLI entry point: every failure ends in the same exit code and message
fn run(args: Args) -> Result<(), Box<dyn Error>> { .. }
```
