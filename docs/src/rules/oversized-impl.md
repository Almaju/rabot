# oversized-impl

**Level**: warn · **Article**: [Structs](https://almaju.github.io/blog/docs/fundamentals/modeling/structs)

> A struct with 25 methods is usually three structs that haven't been
> separated yet.

## What it checks

The inherent impls of one type, in one file, hold more than 20 methods.

## Don't

```rust
impl User {
    fn ban(..) { .. }
    fn charge(..) { .. }
    fn invoice(..) { .. }
    fn notify(..) { .. }
    fn unsubscribe(..) { .. }
    // ... 22 more
}
```

## Do

```rust
impl User { fn ban(..) { .. } .. }
impl BillingProfile { fn charge(..) { .. } fn invoice(..) { .. } .. }
impl NotificationSettings { fn notify(..) { .. } fn unsubscribe(..) { .. } .. }
```

Ask which methods operate on a subset of the fields. That subset is its own
struct, with five methods that actually belong to it.

## Options

```toml
[thresholds]
oversized-impl = 20
```

## Silence it

```rust
// rabot: allow(oversized-impl) builder: one method per option is the whole point
impl CommandBuilder { .. }
```
