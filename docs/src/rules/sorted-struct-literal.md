# sorted-struct-literal

**Level**: warn · **Fixed by** `rabot fmt` (when safe) · **Article**: [Sorting](https://almaju.github.io/blog/docs/fundamentals/style/sorting)

## What it checks

The fields of a struct literal, `User { .. }`, are in alphabetical order,
the same order as the definition. `..base` stays last.

rabot only rewrites a literal when every initializer is plainly side-effect
free: literals, paths, field accesses, references, `Some(..)`, `clone()`,
`Default::default()` and the like. Initializers are evaluated in source
order, so a literal with calls in it is reported and left for you to
reorder by hand.

## Don't

```rust
User { role, name: input.name, id, email: input.email, created_at: now }
```

## Do

```rust
User { created_at: now, email: input.email, id, name: input.name, role }
```

Same order as the struct, every time it is built. A reviewer comparing the
two never has to hunt.

## Silence it

```rust
// rabot: allow(sorted-struct-literal) initializers must run in this order: the token is minted before the session
Session { token: mint(&mut rng), id: next_id(&mut rng) }
```
