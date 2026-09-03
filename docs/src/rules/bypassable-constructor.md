# bypassable-constructor

**Level**: warn · **Article**: [Primitives](https://almaju.github.io/blog/docs/fundamentals/modeling/primitives)

> The invariant lives in the type. It cannot be violated without going
> through the constructor. The constructor rejects violations.

## What it checks

A single-field tuple struct with a `pub` field, in the same file as an
associated function that returns `Result<Self, _>` or `Option<Self>`. The
constructor can say no; the `pub` field lets anyone build the value without
asking.

## Don't

```rust
pub struct Percentage(pub f64);

impl Percentage {
    pub fn new(n: f64) -> Result<Self, ValidationError> {
        if !(0.0..=100.0).contains(&n) { return Err(ValidationError::OutOfRange(n)); }
        Ok(Self(n))
    }
}

let discount = Percentage(250.0); // never went through the door
```

## Do

```rust
pub struct Percentage(f64);

impl Percentage {
    pub fn new(n: f64) -> Result<Self, ValidationError> { .. }

    pub fn value(&self) -> f64 {
        self.0
    }
}
```

One way in. Every `Percentage` in the program has been checked, by
construction, and no function that receives one has to check again.

## Silence it

```rust
// rabot: allow(bypassable-constructor) any f64 is a valid Meters; `parse` only exists for the text form
pub struct Meters(pub f64);
```
