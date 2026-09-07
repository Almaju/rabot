# ambient-config

**Level**: warn · **Article**: [Dependencies](https://almaju.github.io/blog/docs/fundamentals/architecture/dependencies)

> All dependencies get constructed in one place. One file. That's it.

## What it checks

`std::env::var`, `env::var_os` or `env::vars` outside `main` and outside
functions and types that exist to read configuration (a name containing
`config`, `settings` or `env`, such as `load_config` or
`Config::from_env`).

## Don't

```rust
{{#include ambient-config/bad.rs}}
```

The signature says this needs an `App`. It also needs `PORT`, and you find
out when it is missing, at runtime, in the environment where it was not set.

## Do

```rust
{{#include ambient-config/good.rs}}
```

Read once, at the top, parsed into types. Everything below takes what it
needs as a parameter, and a missing variable fails at startup instead of on
the first request that reaches that code path.

## Silence it

```rust
// rabot: allow(ambient-config) RUST_LOG is the logger's own contract, read by the logging crate
```
