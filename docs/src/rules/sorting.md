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

Lists whose order is semantic are never touched: `#[repr]` types, enums with
explicit discriminants, enums deriving `PartialOrd` or `Ord`, and struct
literals whose initializers may have side effects (reported, not rewritten).
Function parameters are never sorted; the article calls calling convention a
real exception.
