# Dependencies

> Every function and class should declare exactly what it needs. No
> singletons. No magic injection. If it's not in the signature, it shouldn't
> exist.

Article: [Dependencies](https://almaju.github.io/blog/docs/fundamentals/architecture/dependencies)

A global is a dependency hidden from every signature that uses it. You find
out about it when two tests mutate it in parallel, or when you try to move
the code and discover what it secretly needed. The compiler cannot help
because the dependency is invisible.

One rule, for the one thing a linter can see: mutable global state.
