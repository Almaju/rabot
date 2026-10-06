// Deliberately unsorted: drop order matters, the guard must release before the pool.
struct Connection {
    pool: Pool,
    guard: Guard,
}
