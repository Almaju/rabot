# commented-out-code

**Level**: warn · **Article**: [Comments](https://almaju.github.io/blog/docs/fundamentals/style/comments)

> You have git. There is no temporary.

## What it checks

A comment block that parses as Rust and carries syntax prose does not use
(`;`, `{`, `()`, `=`, `::`, `->`). Doc comments are never checked; a comment
can legitimately show code.

## Don't

```rust
{{#include commented-out-code/bad.rs}}
```

It creates noise, confuses the reader about what runs, and never gets
cleaned up.

## Do

```rust
{{#include commented-out-code/good.rs}}
```

If you need it back, `git log` exists. If you cannot find it there, you did
not need it.

## Silence it

```rust
// rabot: allow(commented-out-code) the two lines below are the shape of the RFC-12 payload, kept for reference
```
