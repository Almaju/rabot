# sorted-struct-pattern

**Level**: warn · **Fixed by** `rabot fmt` · **Article**: [Sorting](https://almaju.github.io/blog/docs/fundamentals/style/sorting)

## What it checks

Fields in a struct pattern, in `let` or in a `match` arm, are alphabetical.
Patterns have no evaluation order, so this is always safe to rewrite. `..`
stays last.

## Don't

```rust
{{#include sorted-struct-pattern/bad.rs}}
```

## Do

```rust
{{#include sorted-struct-pattern/good.rs}}
```

## Silence it

```rust
// rabot: allow(sorted-struct-pattern) mirrors the wire order documented in RFC-12
```
