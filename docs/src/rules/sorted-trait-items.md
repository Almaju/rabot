# sorted-trait-items

**Level**: warn · **Fixed by** `rabot fmt` · **Article**: [Sorting](https://almaju.github.io/blog/docs/fundamentals/style/sorting)

## What it checks

Items in a `trait` definition: associated consts, then associated types,
then functions, each group alphabetical. Trait impls follow the same order
(see [sorted-impl-items](sorted-impl-items.md)), so a definition and its
impls line up.

## Don't

```rust
{{#include sorted-trait-items/bad.rs}}
```

## Do

```rust
{{#include sorted-trait-items/good.rs}}
```

## Silence it

```rust
// rabot: allow(sorted-trait-items) documented as a state machine: methods appear in call order
trait Handshake { .. }
```
