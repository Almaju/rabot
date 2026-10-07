# Exceptions

> You can break the rule. You must document the exception.

## Say it the way you would anyway

When code breaks a principle on purpose, the honest thing is a comment
saying so. rabot reads that comment:

```rust
// Deliberately unsorted: the guard must release before the channel closes.
struct Connection {
    guard: MutexGuard<'static, ()>,
    channel: Channel,
}
```

A comment that opens with **Deliberately**, **Intentionally**, **On
purpose** or **By design** (any case, `Deliberate:` and `Intentional:` too)
and then gives a reason is an exception. It covers the item that follows it:
the whole struct, the whole function body, the whole impl. As a trailing
comment it covers its own line:

```rust
let port = env::var("PORT").unwrap(); // Intentional: the deploy script validates PORT.
```

The reason can wrap onto the `//` lines below. Unless it says what is
deliberate (below), the comment silences every rabot rule on that item: it
documents a decision about the code, not about a tool.

Nothing in it mentions rabot. That matters in an open-source codebase where
rabot is one contributor's linter rather than the project's: the comment
reads as a plain note to the next maintainer, and nobody has to ask in
review what `rabot` is.

What does not count:

- a marker with no reason: `// Intentionally empty` says the choice was made,
  not why. At least three words have to follow the marker;
- a marker in the middle of a sentence (`// This is intentional ...`) or on
  the second line of a comment;
- doc comments (`///`): they are for the item's users, not its maintainers.

## Say what is deliberate

The words between the marker and the colon can say what the exception is
about, and then it covers that and nothing else. *Deliberately unsorted*
covers the sorting rules; *Intentional unwrap* covers `panic-in-production`:

```rust
let first = items.first().unwrap(); // Intentional unwrap: checked non-empty two lines up.
```

The `PORT` line above names nothing, so it covers both the unwrap and the
environment read. Every word rabot knows, and how it reads them, is on
[Say what is deliberate](deliberate.md).

## Naming the rule

To name rules by their ids instead, or to cover a whole file:

```rust
// rabot: allow(sorted-fields) drop order matters: the guard must release first
```

Several rules at once, and the whole file:

```rust
// rabot: allow(free-function, primitive-soup) FFI surface mirrors the C header
// rabot: allow-file(mock-usage) legacy suite, being replaced under TEST-88
```

## The reason is not optional

A `rabot: allow` comment without a reason is itself reported, at error level, as
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
