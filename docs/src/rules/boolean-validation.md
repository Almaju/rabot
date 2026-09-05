# boolean-validation

**Level**: warn · **Article**: [Errors](https://almaju.github.io/blog/docs/fundamentals/modeling/errors)

> Your type signatures are lying to you.

## What it checks

A function named `validate_*`, `verify_*` or `is_valid*` that returns
`bool`. Plain predicates (`is_empty`, `check_flag`) are not validation and
are left alone. Validation has a reason to say no, and `false` cannot carry
it.

## Don't

```rust
fn validate_email(s: &str) -> bool {
    s.contains('@') && !s.starts_with('@')
}

if !validate_email(&input) {
    return Err(ApiError::Invalid("email".into())); // which rule? the user has to guess
}
```

## Do

```rust
enum EmailError { MissingAt, MissingLocalPart }

impl Email {
    fn parse(s: &str) -> Result<Self, EmailError> {
        if !s.contains('@') { return Err(EmailError::MissingAt); }
        if s.starts_with('@') { return Err(EmailError::MissingLocalPart); }
        Ok(Email(s.to_lowercase()))
    }
}
```

The reason travels with the failure, and the success is a type: nothing
past this line checks the email again.

## Silence it

```rust
// rabot: allow(boolean-validation) a pure predicate used in a filter; there is no caller to inform
fn is_valid_utf8(bytes: &[u8]) -> bool { .. }
```
