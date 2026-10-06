# mock-usage

**Level**: warn · **Article**: [Tests](https://almaju.github.io/blog/docs/fundamentals/architecture/testing)

> Mocks test your assumptions. Real implementations test your code. Those
> are not the same test.

## What it checks

`use mockall`, `faux`, `mry`, `unimock` or `mockall_double`; the
`#[automock]` attribute; the `mock!` macro. This rule is about tests, so it
is not relaxed in test code.

## Don't

```rust
{{#include mock-usage/bad.rs}}
```

The unique constraint on `email` fires in production. The mock never knew
what the database contained, because it was not a database.

## Do

```rust
{{#include mock-usage/good.rs}}
```

Two hours once per dependency. It enforces the same constraints, runs in
milliseconds, and earns its place: local dev, seeding, CI without Docker.

## Silence it

```rust
// rabot: allow-file(mock-usage) legacy suite, replaced by MemGateway under TEST-88
```
