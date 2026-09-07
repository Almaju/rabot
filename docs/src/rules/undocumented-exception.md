# undocumented-exception

**Level**: error

> You can break the rule. You must document the exception.

## What it checks

A `// rabot: allow(..)` or `// rabot: allow-file(..)` comment with nothing
after the parenthesis.

## Don't

```rust
{{#include undocumented-exception/bad.rs}}
```

The comment silences nothing. It is reported as an error, and the rule it
tried to allow still fires.

## Do

```rust
{{#include undocumented-exception/good.rs}}
```

The sentence is the point. The next reader gets the reason instead of a
mystery.
