# The rabot rule catalogue

Every rule traces to a section of [Canon's philosophy](https://almaju.github.io/canon/).
If a rule cannot name its Canon principle, it does not belong here.

**Tier** — `core` (recommended, mechanical), `strict` (fights convention, not the
language), `canonical` (the full doctrine). Each tier includes the ones before it.
**Fix** — ● always auto-fixable, ◐ sometimes, ○ report only.

Rule ids are stable. A rule may gain precision later (see PLAN.md §2.1) but never
a new name.

---

## `order/*` — ordering is never yours to choose

> *"Product fields, union variants, declarations in a file, dispatch arms —
> everything whose order carries no meaning must be in alphabetical order."*

Comparison is **byte-wise and case-sensitive**, so digits < uppercase < lowercase.
All of these are `core` and all are auto-fixable — that is the point of the group.

| id | flags | fix |
|---|---|---|
| `order/struct-fields` | named fields of a `struct`, and of a struct-like enum variant, out of order | ● |
| `order/enum-variants` | variants out of order | ● |
| `order/items` | top-level items out of order within their kind group (`use` → `mod` → `const`/`static` → `type` → `struct`/`enum`/`union` → `trait` → `impl` → `fn`); entry points exempt, `#[cfg(test)] mod tests` pinned last | ● |
| `order/impl-items` | associated items inside an `impl` out of order (consts → types → fns) | ● |
| `order/trait-items` | items inside a `trait` out of order | ● |
| `order/use-decls` | `use` declarations out of order | ● |
| `order/derives` | `#[derive(Debug, Clone)]` — `Clone` sorts first | ● |
| `order/bounds` | `T: Send + Clone`, supertraits, and `where` predicates out of order | ● |
| `order/match-arms` | arms out of order; for literal scrutinees: alphabetical for strings, ascending for ints, catch-all last | ● |
| `order/or-patterns` | `Foo::B \| Foo::A` alternatives out of order | ● |
| `order/struct-literal-fields` | initialisers not in declaration order — fixed only when every value is a path, literal or field access, since reordering evaluation is meaning | ◐ |

**Not ordered, deliberately:** function parameters, call arguments, tuple fields,
array/`vec!` elements, and statement sequences. Position is meaning there, and
Canon never touches operands.

```rust
// bad.rs
#[derive(Debug, Clone)]
enum Reply { Ok(Body), Err(Status) }

// bad.fixed.rs
#[derive(Clone, Debug)]
enum Reply { Err(Status), Ok(Body) }
```

---

## `dispatch/*` — branching is dispatch

> *"The handlers must cover every variant, in the union's order, with no
> wildcard — adding a variant breaks every dispatch that forgot it."*

| id | tier | flags | fix |
|---|---|---|---|
| `dispatch/no-wildcard-arm` | core | `_ => …` on a union scrutinee. A wildcard is how adding a variant stops being a compile error | ○ |
| `dispatch/no-binding-catch-all` | core | `other => …` — the same hole with a name on it | ○ |
| `dispatch/catch-all-last` | core | a literal-dispatch catch-all that is not the final arm | ● |
| `dispatch/no-non-exhaustive` | strict | `#[non_exhaustive]` on a crate-local enum, which forces every downstream dispatch to grow a wildcard | ○ |
| `dispatch/no-guard-fallthrough` | strict | `Foo::A if p => … , Foo::A => …` — a guard splitting one variant into an ordered pair reintroduces sequence-dependent branching | ○ |

Clippy equivalents: `wildcard_enum_match_arm`, `match_wildcard_for_single_variants`.

---

## `form/*` — one spelling per job

> *"There is no `if`/`else` and `match` — there is dispatch. No `while` and `for`
> and recursion — there are collection operations and recursion."*

| id | tier | flags | fix |
|---|---|---|---|
| `form/no-if` | canonical | `if` / `else if` / `else`. Desugars to `match` on the condition with arms `false` then `true` — which is alphabetical, and Canon's `Bool = False + True` | ◐ |
| `form/no-if-let` | strict | `if let P = e { … } else { … }` → `match e { P => …, _ => … }`, then `dispatch/no-wildcard-arm` makes you name the rest | ◐ |
| `form/no-let-else` | strict | `let P = e else { … }` — a third branching form | ◐ |
| `form/no-loop-keywords` | strict | `for`, `while`, `while let`, `loop` → iterator combinators or recursion | ○ |
| `form/no-ufcs` | core | `Vec::len(&v)` where `v.len()` exists — two spellings for one call | ◐ |
| `form/no-as-cast` | core | `x as u64`. Conversion is construction: `From`/`TryFrom`/`u64::from` | ◐ |
| `form/no-nested-use-groups` | core | `use a::{b, c::{self, d}}` → one path per `use`, no braces, no `self` | ● |
| `form/no-shadowing` | strict | rebinding a name in an inner scope, or `let x = x…`. Names lie; a second `x` lies twice | ○ |
| `form/rustfmt-clean` | core | the file differs from `rustfmt` output. Layout is part of the language, not a style choice | ● (delegated) |
| `form/prefer-inclusive-range` | canonical | `a..b` where `a..=b` says it. Canon picks one range convention and keeps it everywhere | ◐ |

```rust
// canonical tier
// bad.rs
if user.is_admin() { grant() } else { deny() }

// bad.fixed.rs
match user.is_admin() {
    false => deny(),
    true => grant(),
}
```

---

## `names/*` — types are the only names

> *"A variable named `userList` that holds a map, a function named `validate`
> that also saves, a parameter named `data`: names lie; types don't."*

| id | tier | flags | fix |
|---|---|---|---|
| `names/duplicate-param-types` | core | two parameters of the same type (`fn f(a: User, b: User)`) — tell them apart with a newtype, which then documents *why* there are two | ○ |
| `names/no-conversion-verbs` | core | `fn to_json`, `fn from_str`, `fn parse_*`, `fn try_new` as free/inherent functions → `From`, `TryFrom`, `FromStr`, `Display`. Conversion is construction | ○ |
| `names/no-getter-prefix` | core | `fn get_name()` → `fn name()` | ● |
| `names/no-vague` | core | `data`, `info`, `tmp`, `val`, `obj`, `thing`, `stuff`, `result` (as a binding), `manager`, `helper`, `handler`, `util`, `misc`, `base`, `common` in any declared name | ○ |
| `names/no-util-modules` | core | `mod utils` / `helpers` / `common` / `misc` — a module named for having no subject | ○ |
| `names/one-type-per-file` | canonical | a module file declaring more than one public type, or a primary type whose name is not the file stem in PascalCase (`http_server.rs` ⇄ `HttpServer`). Canon: the reference *is* the import | ○ |
| `names/single-use-let` | strict | a `let` bound once and read once — inline it into the pipeline | ● |
| `names/no-untyped-let` | canonical | `let x = …` without a type annotation. If the value deserves a name it deserves a type | ○ |
| `names/no-let` | canonical | any `let` in a function body. Values thread through method chains; to name an intermediate, give it a type | ○ |
| `names/no-underscore-binding` | strict | `_x` parameters and bindings — silencing the compiler is a lie about the code | ○ |

---

## `types/*` — if code needs explaining, the fix is a better type

| id | tier | flags | fix |
|---|---|---|---|
| `types/no-bool-param` | core | a `bool` parameter. `Bool = False + True` is a union; a call site reading `f(x, true)` names nothing. Use a two-variant enum | ○ |
| `types/no-tuple-return` | core | returning a tuple of arity ≥ 2 — an unnamed product. Declare the product | ○ |
| `types/no-string-error` | core | `Result<_, String>`, `Result<_, Box<dyn Error>>`, `anyhow::Result` in a library target. `Result` means *failed*, and what failed is a type | ○ |
| `types/no-unit-error` | core | `Result<_, ()>` — an error that carries no information | ○ |
| `types/no-nested-optionality` | core | `Option<Option<T>>`, `Option<Result<…>>`, `Result<Option<…>, _>` in a signature. *"Errors and absence are different things; conflating them is how `null` happened"* | ○ |
| `types/no-option-bool` | core | `Option<bool>` — a three-state value pretending to be two | ○ |
| `types/no-any-downcast` | core | `dyn Any` / `downcast_ref` — a type that has stopped being checked | ○ |
| `types/newtype-primitives` | strict | `String`/`&str`/integer public fields and parameters where a newtype belongs (`Path`, `UserId`, `Port`). Invalid values of a validated type cannot exist — Canon's whole encapsulation story | ○ |
| `types/no-bool-field` | strict | `bool` struct fields — same argument as the parameter rule | ○ |
| `types/evidence-return` | canonical | an effectful function returning `()` or `Result<(), E>`. A write returns `Written`, so a downstream function can demand proof it happened | ○ |

---

## `caps/*` — having a value is having the capability

> *"Dependencies thread explicitly. There are no globals, no singletons, no
> service locators, and no hidden filling-in of an omitted argument."*

The **world boundary** is where capabilities may be minted: `src/main.rs`,
`src/bin/*.rs`, `build.rs`, `tests/`, `benches/`, and `examples/`. Everywhere else
a capability arrives as a parameter or not at all.

| id | tier | flags | fix |
|---|---|---|---|
| `caps/no-ambient-io` | core | `std::fs`, `std::net`, `std::process`, `std::env`, `std::io::{stdin,stdout,stderr}`, `tokio::fs`, `reqwest::{get,blocking}` reached outside the world boundary without the capability being a parameter | ○ |
| `caps/no-ambient-clock` | core | `SystemTime::now`, `Instant::now`, `chrono::Utc::now`, `thread::sleep` — reading the clock is an effect | ○ |
| `caps/no-ambient-random` | core | `rand::random`, `thread_rng`, `getrandom` outside the boundary | ○ |
| `caps/no-globals` | core | `static mut`, `static` holding `Mutex`/`RwLock`/`OnceCell`/`OnceLock`/`Lazy`/`Atomic*`, `lazy_static!`, `thread_local!` — a global is an argument you forgot to pass | ○ |
| `caps/no-process-exit` | core | `process::exit`/`abort` outside the world boundary — a function that ends the program without saying so in its return type | ○ |
| `caps/no-capability-default` | core | an `impl Default` that constructs a client, connection, pool or runtime. *"No hidden filling-in of an omitted argument"* | ○ |
| `caps/no-unsafe` | strict | `unsafe` blocks and functions — the one place the type discipline stops being the sandbox | ○ |
| `caps/no-env-config` | strict | `env::var` anywhere, including the boundary — configuration is an input, and inputs are parameters | ○ |

Clippy/lint equivalents worth pairing: `clippy::disallowed_methods`, `unsafe_code`.

---

## `determinism/*` — two runs produce the same bytes

The same argument as `order/*`, applied to runtime: where the machine would pick
an arbitrary order, pick one.

| id | tier | flags | fix |
|---|---|---|---|
| `determinism/no-hash-iteration` | core | iterating a `HashMap`/`HashSet` (`.iter()`, `for`, `.keys()`, `.values()`, `.collect()` into an ordered type) — use `BTreeMap`/`BTreeSet`, or sort explicitly | ◐ |
| `determinism/prefer-stable-sort` | core | `sort_unstable*` where equal elements are distinguishable | ● |
| `determinism/no-pointer-format` | core | `{:p}` and `as *const _ as usize` in output — addresses differ per run | ○ |
| `determinism/no-hash-in-api` | strict | `HashMap`/`HashSet` in a public signature or serialised type, which exports the nondeterminism | ○ |

---

## `errors/*` — exceptions are `Result` and `?`

| id | tier | flags | fix |
|---|---|---|---|
| `errors/no-unwrap` | core | `unwrap`, `expect`, `unwrap_unchecked` outside `tests/`, `#[cfg(test)]` and the world boundary | ◐ (`?`) |
| `errors/no-panic-macro` | core | `panic!`, `todo!`, `unimplemented!`, `unreachable!` in library code | ○ |
| `errors/no-index-panic` | core | `v[i]`, `s[a..b]` on slices/`Vec`/`String` → `.get(…)`, which returns the `Option` that was always there | ◐ |
| `errors/no-swallow` | core | `let _ = fallible()`, `.ok()` discarding an error, `unwrap_or_default()` on a `Result` — absence and failure conflated and then dropped | ○ |
| `errors/no-error-string-fmt` | strict | building an error with `format!`, which turns a union of causes back into text | ○ |

---

## `structure/*` — the structure is the declaration

> *"There is no `canon.toml` and no lockfile, because the file tree already says
> everything they would."*

| id | tier | flags | fix |
|---|---|---|---|
| `structure/no-mod-rs` | core | `mod.rs` — two spellings for one module, so pick `foo.rs` + `foo/` | ○ |
| `structure/no-inline-mod` | core | `mod foo { … }` with a body, except `#[cfg(test)] mod tests` | ○ |
| `structure/no-glob-import` | core | `use foo::*` — *"ambiguity is a hard error, not a shadowing rule"* | ◐ |
| `structure/no-import-alias` | core | `use foo::Bar as Baz` — a name the compiler cannot check against anything | ○ |
| `structure/no-path-attr` | core | `#[path = "…"]`, which unhooks the module tree from the file tree | ○ |
| `structure/no-formatter-config` | core | a `rustfmt.toml`/`.rustfmt.toml` that sets options. *The formatter has no options* | ● (delete) |
| `structure/no-reexport-facade` | strict | `pub use` re-exports — a second path to one item | ○ |
| `structure/no-feature-branch` | canonical | `#[cfg(feature = …)]` in code, which is 2ⁿ programs in one tree | ○ |

---

## `comments/*` — documentation belongs in types and names

> *"If code needs explaining, the fix is a better type, not prose the compiler
> can't check and the next edit won't update."*

| id | tier | flags | fix |
|---|---|---|---|
| `comments/no-todo-marker` | core | `TODO`, `FIXME`, `HACK`, `XXX`, `WIP` in any comment — an unchecked promise | ○ |
| `comments/no-commented-code` | core | a comment whose body parses as Rust | ● (delete) |
| `comments/no-comments` | strict | any non-doc comment | ● (delete) |
| `comments/no-doc-comments` | canonical | `///`, `//!`, `#[doc]`. The Canon position, stated without softening | ○ |

---

## `meta/*` — there is no escape hatch

| id | tier | flags | fix |
|---|---|---|---|
| `meta/no-suppression` | core | `#[allow(…)]`, `#[expect(…)]`, `#![allow(…)]`, `// rabot: ignore`. A per-line opt-out is a per-line style guide | ○ |
| `meta/no-dead-code-attr` | core | `#[allow(dead_code)]` specifically — dead code is deleted, not annotated | ● (delete the code's annotation, report the code) |

---

## `async/*` — async is a property of types, not syntax

> *"Concurrency is two combinators over the futures you already have — `Parallel`
> fans out, `Race` returns the winner and cancels the loser."*

| id | tier | flags | fix |
|---|---|---|---|
| `async/no-runtime-in-library` | core | `#[tokio::main]`, `Runtime::new`, `block_on` outside the world boundary — the library picking an executor for its caller | ○ |
| `async/no-blocking-in-async` | core | `std::fs`/`std::net`/`thread::sleep`/`block_on` inside an `async fn` | ○ |
| `async/no-detached-spawn` | strict | `spawn` whose handle is dropped → `join!` (Parallel) or `select!` (Race), so the concurrency is in the types | ○ |

---

## Considered and rejected

- **Banning traits.** Canon has none because constructor families replace them;
  Rust has no such replacement, so the rule would have nothing to point at.
- **1-based indexing.** Not reachable from a linter.
- **An abbreviation dictionary** (`ctx`, `req`, `cfg`, …). Endless, arbitrary, and
  the wrong half of the "names lie" argument — the fix is a type, not a longer
  word. `names/no-vague` covers the names that genuinely say nothing.
- **Sorting function parameters.** Position is meaning.
- **A `no-generics` rule.** Generics are Canon's input products, not a
  discretionary choice.
