# Test code

An `unwrap` in a test is the assertion. A `MockClock` under `#[cfg(test)]`
is exactly the injectable the testing article asks for. Test code answers to
a different standard, and rabot knows where it is.

## What counts as test code

- any item or impl item under `#[cfg(test)]`, or a `cfg` that mentions
  `test` (`#[cfg(any(test, feature = "test-utils"))]`)
- `#[test]`, `#[tokio::test]`, `#[rstest]` and `#[bench]` functions
- whole files under `tests/`, `benches/` or `examples/`

## What is relaxed there

The domain rules: `panic-in-production`, `swallowed-error`, `untyped-error`,
`primitive-soup`, `primitive-field`, `stringly-typed-field`,
`bypassable-constructor`, `free-function`, `vague-type-name`,
`orphan-module`, `oversized-impl`, `too-many-parameters`,
`sectioned-function`.

## What is not

Sorting, the comment rules, `mock-usage` and `ignored-test`. A test file is
still code, and the last two are about tests.

## Tuning it

```toml
[tests]
relax = []                            # hold tests to the full standard
relax = ["panic-in-production"]       # relax only this one
```
