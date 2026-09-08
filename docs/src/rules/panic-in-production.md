# panic-in-production

**Level**: warn · **Article**: [Errors](https://almaju.github.io/blog/docs/fundamentals/modeling/errors)

> Every `.unwrap()` in production is a bet that this particular call site
> will never fail. You are wrong about that bet more often than you think,
> and you find out at the worst possible time.

## What it checks

`.unwrap()`, `.expect()`, `.unwrap_err()`, `.expect_err()`, `panic!`,
`unreachable!`, `todo!` and `unimplemented!` outside the two places the
article allows: `fn main`, where a missing config file may legitimately
abort, and test code (see [Test code](../tests.md)).

## Don't

```rust
{{#include panic-in-production/bad.rs}}
```

The signature promises a `User`. It cannot keep that promise, and the caller
has no way to know.

## Do

```rust
{{#include panic-in-production/good.rs}}
```

Startup is the exception: the program cannot run without its config, and
`main` is the one place where panicking is honest.

## Silence it

```rust
let first = items.first().unwrap(); // rabot: allow(panic-in-production) `items` was checked non-empty two lines up
```

An invariant that proves programmer error is the article's other exception.
Say which invariant.
