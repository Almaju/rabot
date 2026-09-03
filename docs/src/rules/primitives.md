# Primitives

> Wrap every primitive that carries domain meaning in a dedicated type. Let
> the compiler enforce what your variable names only suggest.

Article: [Primitives](https://almaju.github.io/blog/docs/fundamentals/modeling/primitives)

In 1999 NASA lost a $327 million orbiter to a `float`: one team measured in
pound-force seconds, another in newton seconds, and the type system had
nothing to say. Your bug will be smaller and the mechanism identical: a raw
value that meant one thing was used where a raw value meaning another was
expected, and nothing caught it.

Four rules push the domain into the type system: two parameters of the same
primitive type, a field whose name promises more than its type, a `String`
that is really an enum, and a validating constructor with a back door.
