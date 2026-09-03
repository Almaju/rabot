# Tests

> Your tests are either fast or honest. Real in-memory implementations are
> both. Mocks are neither.

Article: [Tests](https://almaju.github.io/blog/docs/fundamentals/architecture/testing)

Every test mocked `find_by_email` to return `None`, because that is what the
assertion needed. The unique constraint fired in production. The mock did
exactly what it was told; that is the problem. A `MemDatabase` that enforces
the same constraints in a `HashMap` is fast, honest, and useful outside the
test suite.

Two rules: generated mocks, and tests skipped without a reason.
