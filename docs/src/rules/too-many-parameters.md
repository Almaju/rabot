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
fn render(title: &str, width: u32, height: u32, dpi: u32, margin: u32,
          font: &Font, color: Color, background: Color) -> Image { .. }
```

## Do

```rust
struct Canvas { dpi: u32, height: u32, margin: u32, width: u32 }
struct Style { background: Color, color: Color, font: Font }

fn render(title: &str, canvas: &Canvas, style: &Style) -> Image { .. }
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
