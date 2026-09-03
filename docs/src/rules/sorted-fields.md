# sorted-fields

**Level**: warn · **Fixed by** `rabot fmt` · **Article**: [Sorting](https://almaju.github.io/blog/docs/fundamentals/style/sorting)

> Sort alphabetically. Every time. Object properties, table columns, class
> methods, enum values. Everything that forms a list.

## What it checks

Named fields of a struct, and of struct-like enum variants, are in
alphabetical order (case-insensitive, `field2` before `field10`). Tuple
fields are positional and never sorted. `#[repr(..)]` structs are skipped:
their layout is the point.

## Don't

```rust
struct User {
    id: String,           // primary key first, obviously
    email: String,
    name: String,
    created_at: DateTime, // metadata at the end
    updated_at: DateTime,
    last_login_at: Option<DateTime>,
    phone_number: Option<String>, // where does this go?
}
```

The logic lives in one developer's head. The next developer tacks
`phone_number` at the bottom because that is the safe move. Six developers
later, the struct is sediment.

## Do

```rust
struct User {
    created_at: DateTime,
    email: String,
    id: String,
    last_login_at: Option<DateTime>,
    name: String,
    phone_number: Option<String>,
    updated_at: DateTime,
}
```

Nobody asks where `phone_number` goes. P comes after N, before U.

If two fields belong together, say so with a type, not with proximity:

```rust
struct UserName { first: String, last: String }
struct UserContact { email: String, phone: String }
struct User {
    contact: UserContact,
    name: UserName,
}
```

## Silence it

```rust
// rabot: allow(sorted-fields) drop order matters: the guard must release before the pool
struct Connection {
    guard: MutexGuard<'static, ()>,
    pool: Pool,
}
```

Field order also decides `Debug` output and serde's field order. Those are
rarely a reason; when they are, write them down.
