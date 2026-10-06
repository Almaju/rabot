# Dependencies

> Every function and class should declare exactly what it needs. No
> singletons. No magic injection. If it's not in the signature, it shouldn't
> exist.

Article: [Dependencies](https://almaju.github.io/blog/docs/fundamentals/architecture/dependencies)

A global is a dependency hidden from every signature that uses it. You find
out about it when two tests mutate it in parallel, or when you try to move
the code and discover what it secretly needed. The compiler cannot help
because the dependency is invisible.

A dependency that runs both ways is hidden in a different sense: two
modules that use each other have no boundary between them, only the
appearance of one. You cannot reuse, test or rebind either half alone.

Rules: [`global-state`](global-state.md), [`ambient-config`](ambient-config.md),
[`module-cycle`](module-cycle.md).
