# sorted-variants

**Level**: warn · **Fixed by** `rabot fmt` · **Article**: [Sorting](https://almaju.github.io/blog/docs/fundamentals/style/sorting)

> One rule. Zero documentation required. Culturally neutral.

## What it checks

Enum variants are in alphabetical order. Enums whose variant order is
semantic are skipped without a diagnostic: `#[repr(..)]` enums, enums with
explicit discriminants (`A = 1`), and enums deriving `PartialOrd` or `Ord`,
where declaration order is the comparison order.

## Don't

```rust
enum UserRole {
    Member,
    Admin,
    Guest { since: u64, invited_by: String },
}
```

## Do

```rust
enum UserRole {
    Admin,
    Guest { invited_by: String, since: u64 },
    Member,
}
```

Fields of a struct-like variant are sorted too (that is
[sorted-fields](sorted-fields.md)).

## Silence it

An enum whose order carries meaning usually says so already: derive
`PartialOrd` and rabot steps aside. When the meaning is not a derive, write
it down:

```rust
// rabot: allow(sorted-variants) matches the on-wire protocol numbering
enum Opcode { Connect, Publish, Subscribe, Disconnect }
```
