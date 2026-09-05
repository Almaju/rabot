# dropped-error-context

**Level**: warn · **Article**: [Errors](https://almaju.github.io/blog/docs/fundamentals/modeling/errors)

> Every nested level loses context. Every silent catch is a future 3am.

## What it checks

`.map_err(|_| ..)`, `.or_else(|_| ..)` or `.unwrap_or_else(|_| ..)` whose
closure ignores the error it receives: a parameter named `_` or starting
with `_`. The original failure, the one with the file name and the OS
message, is gone before anyone reads it.

## Don't

```rust
let config = std::fs::read_to_string(path).map_err(|_| ConfigError::Unreadable)?;
```

"Config unreadable." Which file? Permission denied, or not found, or a
directory? The error that knew is gone.

## Do

```rust
enum ConfigError {
    Unreadable { path: PathBuf, #[source] source: std::io::Error },
}

let config = std::fs::read_to_string(&path)
    .map_err(|source| ConfigError::Unreadable { path: path.clone(), source })?;
```

Or with `thiserror`, `#[from]` and `?` do it without a closure at all. The
caller matches on your variant; the log walks `source()` down to the OS.

## Silence it

```rust
// rabot: allow(dropped-error-context) the only possible failure is Utf8; the position is what matters
let name = String::from_utf8(bytes).map_err(|_| NameError::NotUtf8 { offset })?;
```
