# escape-hatch-variant

**Level**: warn · **Article**: [Errors](https://almaju.github.io/blog/docs/fundamentals/modeling/errors)

> Granular enough to make different decisions. If two errors require
> identical handling, they're the same type. If they require different
> recovery strategies, split them.

## What it checks

An enum whose name ends in `Error` with a variant named `Other`, `Unknown`,
`Custom`, `Internal`, `Generic`, `Misc` or `Unexpected`, holding a `String`,
a boxed error, or nothing.

## Don't

```rust
{{#include escape-hatch-variant/bad.rs}}
```

Six months later `Other` carries network timeouts, a provider outage, a
currency mismatch and a typo. The retry logic matches on `CardDeclined` and
guesses at everything else.

## Do

```rust
{{#include escape-hatch-variant/good.rs}}
```

Each variant is a decision the caller can make. Adding a failure mode means
adding a variant, and the compiler finds every `match` that has to decide
about it.

## Options

```toml
[naming]
escape-hatch-variants = ["Custom", "Generic", "Internal", "Misc", "Other", "Unexpected", "Unknown"]
```

## Silence it

```rust
// rabot: allow(escape-hatch-variant) FFI boundary: the C library reports free-form strings we cannot classify
```
