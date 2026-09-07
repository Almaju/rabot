# ignored-test

**Level**: warn · **Article**: [Tests](https://almaju.github.io/blog/docs/fundamentals/architecture/testing)

> The week the senior engineer who understood the mock setup leaves the
> company and nobody knows why three tests are marked `.skip`.

## What it checks

`#[ignore]` without a reason. `#[ignore = "..."]` is the documented
exception and never fires.

## Don't

```rust
{{#include ignored-test/bad.rs}}
```

## Do

```rust
{{#include ignored-test/good.rs}}
```

The reason says whether the test may run again, and who to ask.

## Silence it

Add the reason. That is the fix.
