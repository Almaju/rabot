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
single-line list stays single-line and blank lines keep their place.

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
