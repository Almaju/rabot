# sorted-impl-items

**Level**: warn · **Fixed by** `rabot fmt` · **Article**: [Sorting](https://almaju.github.io/blog/docs/fundamentals/style/sorting)

> Method ordering: constructor, then pub (alpha), then private (alpha).
> Fine. It's documented at the top of the class. Within each section, still
> alphabetical.

## What it checks

Items inside an inherent `impl` follow the article's documented exception:
associated consts, associated types, constructors (associated functions
returning `Self`), `pub` methods, then private methods, each group
alphabetical. Inside `impl Trait for T`, consts, types, then fns.

An impl containing a macro invocation is left alone.

## Don't

```rust
impl UserRepository {
    fn validate(&self) -> bool { .. }
    pub fn delete_user(&self, id: &UserId) { .. }
    pub fn create_user(&self, user: User) { .. }
    pub fn new(db: Database) -> Self { .. }
}
```

## Do

```rust
impl UserRepository {
    pub fn new(db: Database) -> Self { .. }

    pub fn create_user(&self, user: User) { .. }
    pub fn delete_user(&self, id: &UserId) { .. }

    fn validate(&self) -> bool { .. }
}
```

The constructor is where a reader starts. Public API next, in an order that
needs no explaining. Implementation details last.

## Silence it

```rust
// rabot: allow(sorted-impl-items) distance_to and is_near are inseparable: is_near wraps distance_to
impl GpsCoordinates {
    fn distance_to(&self, other: &Self) -> Distance { .. }
    fn is_near(&self, other: &Self, radius: Distance) -> bool { .. }
}
```

The article's own example: one sentence says why they are together. If it
takes more than one sentence, there is a separate type trying to escape.
