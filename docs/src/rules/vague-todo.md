# vague-todo

**Level**: warn · **Article**: [Comments](https://almaju.github.io/blog/docs/fundamentals/style/comments)

> A TODO without context is noise with a timestamp. It will sit there for
> three years, mocking every developer who reads it.

## What it checks

A `TODO`, `FIXME`, `XXX` or `HACK` comment with fewer than 6 words after
the marker and no reference (a ticket like `PERF-112` or `#4521`, or a URL).

## Don't

```rust
{{#include vague-todo/bad.rs}}
```

Refactor what? Why? When?

## Do

```rust
{{#include vague-todo/good.rs}}
```

A known tradeoff and a pointer to the follow-up. That is something the code
cannot say.

## Options

```toml
[thresholds]
vague-todo-min-words = 6
```

## Silence it

Write the sentence. If there is nothing to say, delete the TODO.
