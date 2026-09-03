# mock-usage

**Level**: warn · **Article**: [Tests](https://almaju.github.io/blog/docs/fundamentals/architecture/testing)

> Mocks test your assumptions. Real implementations test your code. Those
> are not the same test.

## What it checks

`use mockall`, `faux`, `mry`, `unimock` or `mockall_double`; the
`#[automock]` attribute; the `mock!` macro. This rule is about tests, so it
is not relaxed in test code.

## Don't

```rust
#[automock]
trait Database {
    fn find_by_email(&self, email: &Email) -> Option<User>;
}

let mut db = MockDatabase::new();
db.expect_find_by_email().return_const(None); // "no duplicate, go ahead"
```

The unique constraint on `email` fires in production. The mock never knew
what the database contained, because it was not a database.

## Do

```rust
struct MemDatabase { users: Mutex<HashMap<UserId, User>> }

impl Database for MemDatabase {
    fn insert(&self, user: NewUser) -> Result<User, DbError> {
        let mut users = self.users.lock()?;
        if users.values().any(|u| u.email == user.email) {
            return Err(DbError::UniqueViolation("email"));
        }
        ..
    }
}
```

Two hours once per dependency. It enforces the same constraints, runs in
milliseconds, and earns its place: local dev, seeding, CI without Docker.

## Silence it

```rust
// rabot: allow-file(mock-usage) legacy suite, replaced by MemGateway under TEST-88
```
