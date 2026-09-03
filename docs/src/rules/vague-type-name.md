# vague-type-name

**Level**: warn · **Article**: [Method Ownership](https://almaju.github.io/blog/docs/fundamentals/modeling/method-ownership)

> When you need a word like "Manager" or "Service" to explain what your code
> does, you're admitting you don't know what your code does.

## What it checks

A struct, enum, trait or type alias whose name ends in `Service`, `Manager`,
`Handler`, `Controller`, `Repository`, `Coordinator`, `Processor`,
`Helper`, `UseCase`, `Util` or `Utils`. The suffix must be a whole word:
`Chandler` is fine.

## Don't

```rust
struct UserService { db: Database }
struct UserRepository { db: Database }
struct UserManager { .. } // added six months ago; nobody knows why
```

You need to ban a user. Which one owns it? You pick one, ship it, and eight
months later there are two `ban_user`s.

## Do

```rust
struct User { .. }
struct Store { .. }

impl User {
    fn ban(self) -> Self { .. }
    async fn save(&self, store: &Store) -> Result<(), SaveError> { .. }
}
```

Tell a colleague what you shipped: "the API and the todos". Those are the
structs. `TodoController` is a name from a tutorial.

## Options

```toml
[naming]
vague-suffixes = ["Controller", "Coordinator", "Handler", "Helper", "Manager",
                  "Processor", "Repository", "Service", "UseCase", "Util", "Utils"]
```

## Silence it

```rust
// rabot: allow(vague-type-name) implements the DDD Repository contract: the domain never sees SQL
struct OrderRepository { .. }
```

If you are genuinely implementing the pattern, own it. Write down why.
