# too-many-parameters

**Level**: warn · **Article**: [Structs](https://almaju.github.io/blog/docs/fundamentals/modeling/structs)

> The signal that you need a struct: you're passing the same three parameters
> to five different functions. Those parameters are trying to tell you
> something.

## What it checks

A function or method takes more than 7 parameters (`self` not counted).
Methods inside `impl Trait for T` are skipped.

## Don't

```rust
{{#include too-many-parameters/bad.rs}}
```

## Do

```rust
{{#include too-many-parameters/good.rs}}
```

Parameters that travel together are a struct waiting to be named. Once
named, they get a home for the logic that was scattered across every
caller.

## Options

```toml
[thresholds]
too-many-parameters = 7
```

## Silence it

```rust
// rabot: allow(too-many-parameters) mirrors the C ABI of libfoo_render exactly
```
