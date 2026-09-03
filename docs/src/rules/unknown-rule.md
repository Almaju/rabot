# unknown-rule

**Level**: error

## What it checks

A `// rabot: allow(..)` comment naming a rule rabot does not have, an
unknown directive after `rabot:`, or a missing closing parenthesis.

## Don't

```rust
// rabot: allow(sort-fields) drop order matters
```

A typo would otherwise be a silent no-op: the comment looks like an
exception and does nothing.

## Do

```rust
// rabot: allow(sorted-fields) drop order matters
```

`rabot rules` lists every name.
