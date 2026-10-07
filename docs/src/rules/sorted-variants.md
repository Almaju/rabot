# sorted-variants

**Level**: warn · **Fixed by** `rabot fmt` · **Article**: [Sorting](https://almaju.github.io/blog/docs/fundamentals/style/sorting)

> One rule. Zero documentation required. Culturally neutral.

## What it checks

Enum variants are in alphabetical order. Enums whose variant order is
behaviour are skipped without a diagnostic: `#[repr(..)]` enums, enums with
explicit discriminants (`A = 1`), enums deriving `PartialOrd` or `Ord`
(declaration order is the comparison order), `#[serde(untagged)]` enums
(serde tries the variants top to bottom), and enums with a derive from
[`order-sensitive-derives`](../configuration.md), such as strum's
`EnumIter`. See [what is left alone](sorting.md#what-is-left-alone).

A blank line starts a new group of variants, sorted on its own; see
[groups](sorting.md#groups).

## Don't

```rust
{{#include sorted-variants/bad.rs}}
```

## Do

```rust
{{#include sorted-variants/good.rs}}
```

Fields of a struct-like variant are sorted too (that is
[sorted-fields](sorted-fields.md)).

## Silence it

An enum whose order carries meaning usually says so already: derive
`PartialOrd` and rabot steps aside. When the meaning is a derive of your
own, add it to `order-sensitive-derives`. When it is not a derive, write it
down:

```rust
// Deliberately unsorted: matches the on-wire protocol numbering.
enum Opcode { Connect, Publish, Subscribe, Disconnect }
```

The comment says the code is deliberate and why, without naming rabot.
To name the rule instead: `// rabot: allow(sorted-variants) <reason>`.
