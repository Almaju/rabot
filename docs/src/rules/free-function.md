# free-function

**Level**: warn · **Article**: [Method Ownership](https://almaju.github.io/blog/docs/fundamentals/modeling/method-ownership)

> A free function almost always has a home. Either in its return type or
> its primary parameter.

## What it checks

A free function (not in an `impl`) whose first parameter is one of your own
types, or which returns one of your own types. `main`, `extern` functions
and generic parameters are excluded.

## Don't

```rust
{{#include free-function/bad.rs}}
```

Six months later somebody who could not find `ban` adds a second one on
`UserService`. Now there are two, and one is wrong.

## Do

```rust
{{#include free-function/good.rs}}
```

`user.ban()`. `Url::parse(s)`. One place to look, one place to add logic,
and the compiler knows the method exists so nobody writes it twice.

## Silence it

```rust
// rabot: allow(free-function) spans two types and belongs to neither: the transaction orchestrates both
fn commit(store: &Store, orders: &[Order]) -> Result<(), CommitError> { .. }
```

The article's exceptions: stateless math (`clamp`), top-level orchestration
(`App`), and operations that genuinely belong to a third thing.
