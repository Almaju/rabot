# sorted-derives

**Level**: warn · **Fixed by** `rabot fmt` · **Article**: [Sorting](https://almaju.github.io/blog/docs/fundamentals/style/sorting)

## What it checks

The list inside `#[derive(..)]` is alphabetical, with one documented
exception: a derive follows the trait it extends. `Eq` comes right after
`PartialEq`, `Ord` right after `PartialOrd`, `Copy` right after `Clone`.
Reading `Eq, PartialEq` backwards is what alphabetical order alone would
give you, and nobody writes it that way.

## Don't

```rust
#[derive(Serialize, Debug, Eq, Clone, PartialEq, Ord, PartialOrd, Copy, Hash)]
struct UserId(String);
```

## Do

```rust
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize)]
struct UserId(String);
```

## Options

Pin derives to a position, in the manner of `cargo-sort-derives`: names
before `"..."` come first in that order, names after it come last, and the
rest sit in between under the rule above. Matching is on the last path
segment, so `serde::Serialize` and `Serialize` are the same pin.

```toml
[sorting]
derive-order = ["Debug", "Clone", "Copy", "...", "Serialize", "Deserialize"]
```

With that setting the example becomes
`#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize)]`.

## Silence it

```rust
// rabot: allow(sorted-derives) the proc macro must see Builder before Default
#[derive(Builder, Default)]
```
