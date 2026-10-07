# Sorting

> Sort your code alphabetically unless you have a documented reason not to.

Article: [Sorting](https://almaju.github.io/blog/docs/fundamentals/style/sorting)

Every developer has a system for where a field goes. The system lives in
their head, conflicts with everyone else's, and is invisible to the next
person. Six developers later the struct is sediment: layers, each one a
person who did not want to argue. Alphabetical order needs no documentation,
no politics and no archaeology. You do not scan; you binary-search.

These seven rules are the formatter half of rabot. `rabot fmt` rewrites all
of them in place; `rabot check` reports them.

## What is sorted, and how

Order is case-insensitive and natural, so `field2` comes before `field10`.
Comments before a member move with it. Whitespace stays where it is, so a
single-line list stays single-line.

## Groups

Alphabetical order removes the invisible system: the one in a developer's
head that nobody else can read. Grouping related fields is not that
system, as long as the grouping is visible. A blank line makes it visible:

```rust
struct Person {
    // Name
    first_name: String,
    last_name: String,

    // Address
    city: String,
    street: String,
}
```

A blank line splits fields, variants, struct literals and struct patterns
into groups, and rabot sorts each group on its own. Nothing moves across a
blank line. A comment at the top of a group is the group's heading and stays
there; any other comment moves with the member below it. To document the
first field of a group rather than head the group, use a doc comment
(`///`), which belongs to the field.

Two things are not groups:

- A list where *every* member is set apart by a blank line. That is
  spacing (a blank line between each documented field, say), and the list is
  sorted as a whole. Otherwise the style alone would switch the rule off.
- Impl and trait items. A blank line between every method is the norm
  there, so they are always sorted as a whole.

Alphabetical order still applies inside a group, so `start, end` or
`x, y, z` in one group are reported. When their order matters, they are
often a type waiting to be named (`Range`, `Point`); when not, write the
reason down.

## What is left alone

Lists whose order is behaviour are never touched, and need no comment:

| What | Why the order matters |
| --- | --- |
| `#[repr(..)]` types | the memory layout follows declaration order |
| types deriving `PartialOrd` or `Ord` | comparison goes field by field, variant by variant, in declaration order |
| enums with explicit discriminants (`A = 1`) | the numbering is the point |
| `#[serde(untagged)]` enums, or enums with an untagged variant | serde tries the variants top to bottom and keeps the first that fits |
| types with a derive from [`order-sensitive-derives`](../configuration.md) | by default: clap, argh and structopt (positional arguments), borsh, bincode and SCALE (`Encode`/`Decode`, positional binary encodings), strum (`EnumIter`, `VariantArray`, `VariantNames`), uniffi (`Record`, `Enum`, `Error`: foreign constructors) |

Attributes inside `#[cfg_attr(..)]` count too: a type that derives
`uniffi::Record` only behind a feature still keeps its order.

Struct literals whose initializers may have side effects are reported, not
rewritten. Function parameters are never sorted; the article calls calling
convention a real exception.

What rabot cannot see is drop order: fields are dropped top to bottom, and
a guard that must outlive what it protects is a reason to write down.
