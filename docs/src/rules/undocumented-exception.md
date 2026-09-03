# undocumented-exception

**Level**: error

> You can break the rule. You must document the exception.

## What it checks

A `// rabot: allow(..)` or `// rabot: allow-file(..)` comment with nothing
after the parenthesis.

## Don't

```rust
// rabot: allow(sorted-fields)
struct Connection { guard: Guard, pool: Pool }
```

The comment silences nothing. It is reported as an error, and the rule it
tried to allow still fires.

## Do

```rust
// rabot: allow(sorted-fields) drop order matters: the guard must release before the pool
struct Connection { guard: Guard, pool: Pool }
```

The sentence is the point. The next reader gets the reason instead of a
mystery.
