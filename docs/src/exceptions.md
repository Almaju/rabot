# Exceptions

> You can break the rule. You must document the exception.

Every rule can be silenced for one item with a comment that names the rule
and says why:

```rust
// rabot: allow(sorted-fields) drop order matters: the guard must release first
struct Connection {
    guard: MutexGuard<'static, ()>,
    channel: Channel,
}
```

The comment covers the item that follows it: the whole struct, the whole
function body, the whole impl. As a trailing comment it covers its own line:

```rust
let port = env::var("PORT").unwrap(); // rabot: allow(panic-in-production) validated by the deploy script
```

Several rules at once, and the whole file:

```rust
// rabot: allow(free-function, primitive-soup) FFI surface mirrors the C header
// rabot: allow-file(mock-usage) legacy suite, being replaced under TEST-88
```

## The reason is not optional

An allow comment without a reason is itself reported, at error level, as
[`undocumented-exception`](rules/undocumented-exception.md). A rule name
rabot does not know is [`unknown-rule`](rules/unknown-rule.md). The point of
the comment is the sentence after the parenthesis: the next reader, or you in
six months, gets the reason instead of a mystery.

## Turning a rule off everywhere

When a rule does not apply to a project at all, the config is the place, and
the config file is the documentation:

```toml
[rules]
free-function = "allow"   # a crate of pure math functions
```
