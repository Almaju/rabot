# primitive-soup

**Level**: warn · **Article**: [Primitives](https://almaju.github.io/blog/docs/fundamentals/modeling/primitives)

> Three bugs. All type-safe. The compiler is happy. Your users are not.

## What it checks

A function takes two or more parameters of the same primitive type
(`String`, `&str`, integers, floats, `bool`, `Option<..>` of those). Two
`String` parameters can be swapped at any call site and the program still
compiles. Methods inside `impl Trait for T` are skipped: that signature is
the trait's.

## Don't

```rust
fn send_invoice(user_id: String, email: String, invoice_id: String) {
    send_email(&user_id, &invoice_id); // swapped
    log_access(&invoice_id, &email);   // wrong order
}
```

## Do

```rust
struct UserId(String);
struct Email(String);
struct InvoiceId(String);

fn send_invoice(user_id: UserId, email: Email, invoice_id: InvoiceId) {
    send_email(&user_id, &invoice_id); // compile error
    log_access(&invoice_id, &email);   // compile error
}
```

You write the type once. The build catches the swap instead of the review.
The newtype compiles to the same representation as the primitive; the cost
is zero.

## Options

```toml
[thresholds]
primitive-soup = 2   # parameters of the same primitive type before it fires
```

## Silence it

```rust
// rabot: allow(primitive-soup) stateless math with no subject: min, max are both just numbers
fn clamp(value: f64, min: f64, max: f64) -> f64 { .. }
```

The article's own exception: genuinely stateless math, where no parameter
means anything on its own.
