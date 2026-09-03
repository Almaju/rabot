# sorted-trait-items

**Level**: warn · **Fixed by** `rabot fmt` · **Article**: [Sorting](https://almaju.github.io/blog/docs/fundamentals/style/sorting)

## What it checks

Items in a `trait` definition: associated consts, then associated types,
then functions, each group alphabetical. Trait impls follow the same order
(see [sorted-impl-items](sorted-impl-items.md)), so a definition and its
impls line up.

## Don't

```rust
trait Persist {
    fn save(&self, store: &Store) -> Result<(), SaveError>;
    type Error;
    fn load(id: &Id, store: &Store) -> Result<Self, Self::Error>;
    const TABLE: &'static str;
}
```

## Do

```rust
trait Persist {
    const TABLE: &'static str;
    type Error;
    fn load(id: &Id, store: &Store) -> Result<Self, Self::Error>;
    fn save(&self, store: &Store) -> Result<(), SaveError>;
}
```

## Silence it

```rust
// rabot: allow(sorted-trait-items) documented as a state machine: methods appear in call order
trait Handshake { .. }
```
