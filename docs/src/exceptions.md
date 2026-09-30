# Exceptions

> You can break the rule. You must document the exception.

Every rule can be silenced for one item with a comment that names the rule
and says why:

```rust
// allow(sorted-fields) drop order matters: the guard must release first
struct Connection {
    guard: MutexGuard<'static, ()>,
    channel: Channel,
}
```

The comment covers the item that follows it: the whole struct, the whole
function body, the whole impl. As a trailing comment it covers its own line:

```rust
let port = env::var("PORT").unwrap(); // allow(panic-in-production) validated by the deploy script
```

Several rules at once, and the whole file:

```rust
// allow(free-function, primitive-soup) FFI surface mirrors the C header
// allow-file(mock-usage) legacy suite, being replaced under TEST-88
```

## No tool name in the code

The comment names the rule and the reason, not the tool. rabot is often run
by one contributor on a codebase that has not adopted it; a
`// allow(panic-in-production) the lock is never poisoned` there reads as an
ordinary note about the code, and nobody has to explain what rabot is in
review.

A bare `// allow(..)` is an allow comment only when it names at least one
rabot rule, so comments meant for other tools or other people are left
alone. When you want the directive to be unambiguous, prefix it:

```rust
// rabot: allow(sorted-fields) drop order matters: the guard must release first
```

Both forms silence the same things. The prefixed form is always a
directive, so a misspelled lone rule name in it is reported as
[`unknown-rule`](rules/unknown-rule.md); in the bare form it is just a
comment.

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
