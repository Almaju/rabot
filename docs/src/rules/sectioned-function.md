# sectioned-function

**Level**: warn · **Article**: [Comments](https://almaju.github.io/blog/docs/fundamentals/style/comments)

> You have three functions trapped inside one. The comment is an informal
> table of contents for code that should have been split. The section
> headers become function names.

## What it checks

A function body containing 3 or more leading comment blocks (a comment on
its own line, introducing the code below it). Trailing comments beside
code, `TODO`/`FIXME`/`SAFETY` notes and ticket links never count.

## Don't

```rust
fn process(order: &mut Order) -> Result<(), ProcessError> {
    // step 1: validate
    if order.items.is_empty() { return Err(ProcessError::Empty); }
    // step 2: transform
    let total = order.items.iter().map(Item::price).sum();
    // step 3: persist
    store.save(order, total)
}
```

## Do

```rust
fn process(order: &mut Order) -> Result<(), ProcessError> {
    order.validate()?;
    let total = order.total();
    store.save(order, total)
}
```

Each header became a name. The function reads as the summary the comments
were trying to be, and each piece can be tested on its own.

## Options

```toml
[thresholds]
section-comments = 3
```

## Silence it

```rust
// rabot: allow(sectioned-function) the protocol handshake is documented step by step against RFC 6455 §4
fn handshake(..) { .. }
```
