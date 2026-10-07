# module-cycle

**Level**: warn · **Article**: [Dependencies](https://almaju.github.io/blog/docs/fundamentals/architecture/dependencies)

> Layers point one way. If the crate exposes A, and A uses B, then B does not
> use A.

## What it checks

Two modules of the same crate that depend on each other, directly or through
others: `ffi` uses `worker`, and `worker` uses a type from `ffi`. A
dependency is a path into another module of the crate (`use crate::a::B`,
`super::B`, `self::a::B`, or `a::B` for a child module `a`). Re-exports are
followed to where the item is defined, so `crate::ApiResult` counts as a
dependency on `ffi` when the root only re-exports it with `pub use ffi::*`.

Modules are compared with their siblings: `ffi::error` depending on `worker`
is `ffi` depending on `worker`. A module's own items count as a sibling of its
children, so a crate root that uses `worker` while `worker` uses a type alias
defined in `lib.rs` is a cycle too. A re-export (`pub use`) is not a
dependency of the module that re-exports: exposing a module is not using it.
Test code is left out.

The rule reads the whole crate, following `mod` declarations from `src/lib.rs`,
`src/main.rs` and `src/bin/`, even when only some files are being checked. It
reports each dependency that closes a cycle, so both ends of a two-module
cycle get a diagnostic.

## Don't

```rust
{{#include module-cycle/bad.rs}}
```

`worker` cannot be used, tested or bound again (to WASM, to a CLI) without
dragging the FFI layer along, and the FFI layer is no longer a thin
translation: it now holds a type the engine is built on. Nothing marks where
one ends and the other begins.

## Do

```rust
{{#include module-cycle/good.rs}}
```

The lower layer returns its own types; the upper one wraps them. Dependencies
point one way, and each layer can be understood, and replaced, from below.

## Silence it

An exception on the dependency you accept removes it from the graph, which
breaks the cycle for both ends:

```rust
// Deliberate, until ADS-231 moves it: the callback registry is shared with the FFI layer.
use crate::ffi::CallbackRegistry;
```
