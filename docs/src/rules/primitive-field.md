# primitive-field

**Level**: warn · **Article**: [Primitives](https://almaju.github.io/blog/docs/fundamentals/modeling/primitives)

> Data comes in as primitives. It leaves as domain types. It stays as domain
> types until it leaves the system. No exceptions.

## What it checks

A named struct field whose name says "domain concept" and whose type says
"anything": `email: String`, `user_id: u64`, `latitude: f64`,
`price: Option<f64>`. The names come from a configurable list of words and
`_`-suffixes; `bool` and `char` never count.

Wire shapes are skipped. A struct named `*Request`, `*Response`, `*Row`,
`*Dto` and so on is where primitives arrive, and parsing happens right after.

## Don't

```rust
struct User {
    email: String,
    id: String,
    latitude: f64,
}
```

`email` accepts `"not an email"`. `id` accepts an order id. `latitude`
accepts 400.

## Do

```rust
struct User {
    email: Email,
    id: UserId,
    latitude: Latitude,
}

impl Email {
    fn parse(s: &str) -> Result<Self, ValidationError> { .. }
}
```

Validate once, at construction, at the boundary. Everything past that line
is typed and nobody checks again.

## Options

```toml
[naming]
domain-fields = ["_id", "amount", "email", "latitude", "longitude", "password",
                 "phone", "price", "token", "url"]
boundary-suffixes = ["Body", "Dto", "Params", "Payload", "Query", "Record",
                     "Request", "Response", "Row"]
```

## Silence it

```rust
// rabot: allow(primitive-field) mirrors the vendor's CSV columns; parsed into Reading right after
struct RawReading { latitude: f64, longitude: f64 }
```
