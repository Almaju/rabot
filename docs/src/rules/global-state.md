# global-state

**Level**: warn · **Article**: [Dependencies](https://almaju.github.io/blog/docs/fundamentals/architecture/dependencies)

> Invisible dependencies are the cockroaches of software architecture:
> everywhere, impossible to count, surviving every refactor.

## What it checks

A `static` with interior mutability (`Mutex`, `RwLock`, `OnceLock`,
`OnceCell`, `LazyLock`, `Cell`, `RefCell`, atomics), `static mut`, or a
`lazy_static!` block. Statics whose name contains `LOG` are exempt by
default: a logger is infrastructure nobody swaps in tests.

## Don't

```rust
{{#include global-state/bad.rs}}
```

Zero constructor parameters. Looks simple. Until two tests run in parallel
against the same global, or you need to point it at another database.

## Do

```rust
{{#include global-state/good.rs}}
```

Exactly as complex as it actually is, and checked at compile time.

## Options

```toml
[global-state]
allowed-names = ["LOG"]   # substring match, case-insensitive
```

## Silence it

```rust
// rabot: allow(global-state) compiled-once regex; a pure value, never swapped
static EMAIL: LazyLock<Regex> = LazyLock::new(|| Regex::new(..).unwrap());
```
