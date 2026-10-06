# undocumented-exception

**Level**: error

> You can break the rule. You must document the exception.

## What it checks

A `// rabot: allow(..)` or `// rabot: allow-file(..)` comment with nothing
after the parenthesis.

A plain comment such as `// Intentionally empty` is not reported: it never
became an exception, because a marker (*Deliberately*, *Intentionally*, *On
purpose*, *By design*) needs a reason after it (see
[Exceptions](../exceptions.md)). The rule it meant to silence still fires.

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

Say it the way you would to the next maintainer: the code is deliberate,
and here is why. The comment does not mention rabot, and the sentence is the
point: the next reader gets the reason instead of a mystery. Naming the rule
works too, as long as the reason follows:
`// rabot: allow(sorted-fields) drop order matters`.
