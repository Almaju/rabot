# syntax-error

**Level**: error

## What it checks

A file that does not parse as Rust. rabot reports it and moves on to the
next file, so a broken file never silently disables checking, and never
gets rewritten by `rabot fmt`.

## What to do

Fix the file; `cargo check` says where. If it is generated or vendored code
that is not meant to parse on its own, exclude it:

```toml
[files]
exclude = ["src/generated"]
```
