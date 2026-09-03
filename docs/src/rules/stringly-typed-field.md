# stringly-typed-field

**Level**: warn · **Article**: [Primitives](https://almaju.github.io/blog/docs/fundamentals/modeling/primitives)

> The comment `// status: 'pending' | 'approved' | 'rejected'` is a `Status`
> enum that hasn't been written yet. Write it. Delete the comment.

## What it checks

A named field called `status`, `state`, `kind`, `role`, `mode`, `level`,
`phase`, `stage`, `category` or `type` whose type is `String`, `&str` or
`Option<String>`. A value with a handful of valid spellings is an enum, and
the compiler checks every `match` on an enum.

Wire shapes (`*Request`, `*Row`, ...) are skipped, like
[primitive-field](primitive-field.md).

## Don't

```rust
struct Order {
    status: String, // "pending" | "approved" | "rejected"
}

if order.status == "aproved" { ship(order) } // never ships
```

## Do

```rust
enum OrderStatus { Approved, Pending, Rejected }

struct Order {
    status: OrderStatus,
}

match order.status {
    OrderStatus::Approved => ship(order),
    OrderStatus::Pending | OrderStatus::Rejected => {}
}
```

Parse the string once, where it enters. A typo is now a compile error, and
adding a variant makes every `match` that forgot it fail to build.

## Options

```toml
[naming]
enum-fields = ["category", "kind", "level", "mode", "phase", "role", "stage", "state", "status"]
```

## Silence it

```rust
// rabot: allow(stringly-typed-field) free-form user label, not a closed set
struct Tag { kind: String }
```
