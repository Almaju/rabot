# commented-out-code

**Level**: warn · **Article**: [Comments](https://almaju.github.io/blog/docs/fundamentals/style/comments)

> You have git. There is no temporary.

## What it checks

A comment block that parses as Rust and carries syntax prose does not use
(`;`, `{`, `()`, `=`, `::`, `->`). Doc comments are never checked; a comment
can legitimately show code.

## Don't

```rust
fn total(items: &[Item]) -> Money {
    // let discount = apply_coupon(&items);
    // items.iter().map(|i| i.price - discount).sum()
    items.iter().map(|i| i.price).sum()
}
```

It creates noise, confuses the reader about what runs, and never gets
cleaned up.

## Do

```rust
fn total(items: &[Item]) -> Money {
    items.iter().map(|i| i.price).sum()
}
```

If you need it back, `git log` exists. If you cannot find it there, you did
not need it.

## Silence it

```rust
// rabot: allow(commented-out-code) the two lines below are the shape of the RFC-12 payload, kept for reference
```
