# sorted-fields

**Level**: warn · **Fixed by** `rabot fmt` · **Article**: [Sorting](https://almaju.github.io/blog/docs/fundamentals/style/sorting)

> Sort alphabetically. Every time. Object properties, table columns, class
> methods, enum values. Everything that forms a list.

## What it checks

Named fields of a struct, and of struct-like enum variants, are in
alphabetical order (case-insensitive, `field2` before `field10`). Tuple
fields are positional and never sorted.

Types whose field order is behaviour are skipped without a diagnostic:
`#[repr(..)]` types (the layout is the point), types deriving `PartialOrd`
or `Ord` (comparison goes field by field), and types with a derive from
[`order-sensitive-derives`](../configuration.md): clap's positional
arguments, uniffi's generated constructors, positional binary encodings.
See [what is left alone](sorting.md#what-is-left-alone).

## Don't

```rust
{{#include sorted-fields/bad.rs}}
```

The logic lives in one developer's head. The next developer tacks
`phone_number` at the bottom because that is the safe move. Six developers
later, the struct is sediment.

## Do

```rust
{{#include sorted-fields/good.rs}}
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
// Deliberately unsorted: drop order matters, the guard must release before the pool.
struct Connection {
    guard: MutexGuard<'static, ()>,
    pool: Pool,
}
```

The comment says the code is deliberate and why, without naming rabot.
To name the rule instead: `// rabot: allow(sorted-fields) <reason>`.

Field order also decides `Debug` output and serde's field order. Those are
rarely a reason; when they are, write them down. When a derive of yours
reads field order (a binary encoding, code generation), add it to
`order-sensitive-derives` instead, and every type deriving it is left alone.
