// rabot: allow(sorted-fields) drop order matters: the guard must release before the pool
struct Connection {
    guard: Guard,
    pool: Pool,
}
