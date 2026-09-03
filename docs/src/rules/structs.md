# Structs and method ownership

> Obsess over your data structures, not your algorithms. Put behavior on the
> type it belongs to. Repository, Handler, and Service are names for
> decisions you haven't made yet.

Articles: [Structs](https://almaju.github.io/blog/docs/fundamentals/modeling/structs),
[Method Ownership](https://almaju.github.io/blog/docs/fundamentals/modeling/method-ownership)

You use `HashMap` every day without caring about its bucket layout. Your job
is to build the same thing for your domain: a `GpsCoordinates` with a
`distance_to`, not twelve copies of the Haversine formula with four `f64`
parameters each. Every function in `utils.rs` is a method on a type that
does not exist yet, and every `UserService` is a place where the same rule
gets implemented twice.

Five rules: free functions that belong on a type, names that dodge a
decision, drawer modules, impls that are three types in a trenchcoat, and
parameter lists that are a struct waiting to be named.
