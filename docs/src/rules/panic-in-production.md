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
fn get_user(id: &UserId, db: &Database) -> User {
    db.query("SELECT ...", id).unwrap()
}
```

The signature promises a `User`. It cannot keep that promise, and the caller
has no way to know.

## Do

```rust
fn get_user(id: &UserId, db: &Database) -> Result<User, NotFoundError> {
    db.query("SELECT ...", id).ok_or(NotFoundError { user_id: id.clone() })
}
```

```rust
// Startup: the program cannot run without these. Panicking is honest here.
fn main() {
    let config = load_config().expect("config file required for startup");
}
```

## Silence it

```rust
let first = items.first().unwrap(); // rabot: allow(panic-in-production) `items` was checked non-empty two lines up
```

An invariant that proves programmer error is the article's other exception.
Say which invariant.
