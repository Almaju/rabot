# unknown-rule

**Level**: error

## What it checks

An allow comment naming a rule rabot does not have, an unknown directive
after `rabot:`, or a missing closing parenthesis.

The bare form, `// allow(..)`, is only an allow comment when it names at
least one rabot rule; otherwise it is somebody else's comment and is left
alone. So a bare comment whose only rule is misspelled goes unnoticed, while
`// allow(sorted-fields, sort-variants)` reports the second name. The
`// rabot: allow(..)` form is always a directive, and every mistake in it is
reported.

## Don't

```rust
{{#include unknown-rule/bad.rs}}
```

A typo would otherwise be a silent no-op: the comment looks like an
exception and does nothing.

## Do

```rust
{{#include unknown-rule/good.rs}}
```

`rabot rules` lists every name.
