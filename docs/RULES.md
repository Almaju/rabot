# The rabot rule catalogue

Every rule traces to a section of [Canon's philosophy](https://almaju.github.io/canon/).
If a rule cannot name its Canon principle, it does not belong here.

**There is one tier, and it is canonical.** Every rule below is a violation, not a
suggestion — there are no levels, no severities, no profiles, and no way to turn a
rule off. A rule is either in this catalogue or it does not exist. That makes rabot
unusable on a codebase written without it, which is the honest consequence of a
maximally opinionated tool and not a defect to be configured away.

**Fix** — ● always auto-fixable, ◐ sometimes, ○ report only.

Rule ids are stable. A rule may gain precision later (see PLAN.md §2.1) but never
a new name.

---

## `order/*` — ordering is never yours to choose

> *"Product fields, union variants, declarations in a file, dispatch arms —
> everything whose order carries no meaning must be in alphabetical order."*

Comparison is **byte-wise and case-sensitive**, so digits < uppercase < lowercase.
Every rule here is auto-fixable — that is the point of the group.

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

| id | flags | fix |
|---|---|---|
| `dispatch/no-wildcard-arm` | `_ => …` on a union scrutinee. A wildcard is how adding a variant stops being a compile error | ○ |
| `dispatch/no-binding-catch-all` | `other => …` — the same hole with a name on it | ○ |
| `dispatch/catch-all-last` | a literal-dispatch catch-all that is not the final arm | ● |
| `dispatch/no-non-exhaustive` | `#[non_exhaustive]` on a crate-local enum, which forces every downstream dispatch to grow a wildcard | ○ |
| `dispatch/no-guard-fallthrough` | `Foo::A if p => … , Foo::A => …` — a guard splitting one variant into an ordered pair reintroduces sequence-dependent branching | ○ |

Clippy equivalents: `wildcard_enum_match_arm`, `match_wildcard_for_single_variants`.

---

## `form/*` — one spelling per job

> *"There is no `if`/`else` and `match` — there is dispatch. No `while` and `for`
> and recursion — there are collection operations and recursion."*

| id | flags | fix |
|---|---|---|
| `form/no-if` | `if` / `else if` / `else`. Desugars to `match` on the condition with arms `false` then `true` — which is alphabetical, and Canon's `Bool = False + True` | ◐ |
| `form/no-if-let` | `if let P = e { … } else { … }` → `match e { P => …, _ => … }`, then `dispatch/no-wildcard-arm` makes you name the rest | ◐ |
| `form/no-let-else` | `let P = e else { … }` — a third branching form | ◐ |
| `form/no-loop-keywords` | `for`, `while`, `while let`, `loop` → iterator combinators or recursion | ○ |
| `form/no-ufcs` | `Vec::len(&v)` where `v.len()` exists — two spellings for one call | ◐ |
| `form/no-as-cast` | `x as u64`. Conversion is construction: `From`/`TryFrom`/`u64::from` | ◐ |
| `form/no-nested-use-groups` | `use a::{b, c::{self, d}}` → one path per `use`, no braces, no `self` | ● |
| `form/no-shadowing` | rebinding a name in an inner scope, or `let x = x…`. Names lie; a second `x` lies twice | ○ |
| `form/rustfmt-clean` | the file differs from `rustfmt` output. Layout is part of the language, not a style choice | ● (delegated) |
| `form/prefer-inclusive-range` | `a..b` where `a..=b` says it. Canon picks one range convention and keeps it everywhere | ◐ |

```rust
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

| id | flags | fix |
|---|---|---|
| `names/no-let` | any `let` in a function body. Values thread through method chains; to name an intermediate, give it a type. The most aggressive rule in the catalogue and the one most directly from Canon | ○ |
| `names/no-untyped-let` | `let x = …` without a type annotation — reported separately so the diagnostic can say which half is wrong while `names/no-let` is being adopted | ○ |
| `names/single-use-let` | a `let` bound once and read once — inline it into the pipeline | ● |
| `names/duplicate-param-types` | two parameters of the same type (`fn f(a: User, b: User)`) — tell them apart with a newtype, which then documents *why* there are two | ○ |
| `names/no-conversion-verbs` | `fn to_json`, `fn from_str`, `fn parse_*`, `fn try_new` as free/inherent functions → `From`, `TryFrom`, `FromStr`, `Display`. Conversion is construction | ○ |
| `names/no-getter-prefix` | `fn get_name()` → `fn name()` | ● |
| `names/no-vague` | `data`, `info`, `tmp`, `val`, `obj`, `thing`, `stuff`, `result` (as a binding), `manager`, `helper`, `handler`, `util`, `misc`, `base`, `common` in any declared name | ○ |
| `names/no-util-modules` | `mod utils` / `helpers` / `common` / `misc` — a module named for having no subject | ○ |
| `names/one-type-per-file` | a module file declaring more than one public type, or a primary type whose name is not the file stem in PascalCase (`http_server.rs` ⇄ `HttpServer`). Canon: the reference *is* the import | ○ |
| `names/no-underscore-binding` | `_x` parameters and bindings — silencing the compiler is a lie about the code | ○ |

---

## `types/*` — if code needs explaining, the fix is a better type

| id | flags | fix |
|---|---|---|
| `types/no-bool-param` | a `bool` parameter. `Bool = False + True` is a union; a call site reading `f(x, true)` names nothing. Use a two-variant enum | ○ |
| `types/no-bool-field` | `bool` struct fields — same argument as the parameter rule | ○ |
| `types/no-tuple-return` | returning a tuple of arity ≥ 2 — an unnamed product. Declare the product | ○ |
| `types/no-string-error` | `Result<_, String>`, `Result<_, Box<dyn Error>>`, `anyhow::Result` in a library target. `Result` means *failed*, and what failed is a type | ○ |
| `types/no-unit-error` | `Result<_, ()>` — an error that carries no information | ○ |
| `types/no-nested-optionality` | `Option<Option<T>>`, `Option<Result<…>>`, `Result<Option<…>, _>` in a signature. *"Errors and absence are different things; conflating them is how `null` happened"* | ○ |
| `types/no-option-bool` | `Option<bool>` — a three-state value pretending to be two | ○ |
| `types/no-any-downcast` | `dyn Any` / `downcast_ref` — a type that has stopped being checked | ○ |
| `types/newtype-primitives` | `String`/`&str`/integer public fields and parameters where a newtype belongs (`Path`, `UserId`, `Port`). Invalid values of a validated type cannot exist — Canon's whole encapsulation story | ○ |
| `types/evidence-return` | an effectful function returning `()` or `Result<(), E>`. A write returns `Written`, so a downstream function can demand proof it happened | ○ |

---

## `caps/*` — having a value is having the capability

> *"Dependencies thread explicitly. There are no globals, no singletons, no
> service locators, and no hidden filling-in of an omitted argument."*

The **world boundary** is where capabilities may be minted: `src/main.rs`,
`src/bin/*.rs`, `build.rs`, `tests/`, `benches/`, and `examples/`. Everywhere else
a capability arrives as a parameter or not at all. The boundary is fixed by those
paths, not configurable — Canon selects the world by shape, so rabot selects it by
position in the tree.

| id | flags | fix |
|---|---|---|
| `caps/no-ambient-io` | `std::fs`, `std::net`, `std::process`, `std::env`, `std::io::{stdin,stdout,stderr}`, `tokio::fs`, `reqwest::{get,blocking}` reached outside the world boundary without the capability being a parameter | ○ |
| `caps/no-ambient-clock` | `SystemTime::now`, `Instant::now`, `chrono::Utc::now`, `thread::sleep` — reading the clock is an effect | ○ |
| `caps/no-ambient-random` | `rand::random`, `thread_rng`, `getrandom` outside the boundary | ○ |
| `caps/no-globals` | `static mut`, `static` holding `Mutex`/`RwLock`/`OnceCell`/`OnceLock`/`Lazy`/`Atomic*`, `lazy_static!`, `thread_local!` — a global is an argument you forgot to pass | ○ |
| `caps/no-process-exit` | `process::exit`/`abort` outside the world boundary — a function that ends the program without saying so in its return type | ○ |
| `caps/no-capability-default` | an `impl Default` that constructs a client, connection, pool or runtime. *"No hidden filling-in of an omitted argument"* | ○ |
| `caps/no-unsafe` | `unsafe` blocks and functions — the one place the type discipline stops being the sandbox | ○ |
| `caps/no-env-config` | `env::var` anywhere, including the boundary — configuration is an input, and inputs are parameters | ○ |

Clippy/lint equivalents worth pairing: `clippy::disallowed_methods`, `unsafe_code`.

---

## `determinism/*` — two runs produce the same bytes

The same argument as `order/*`, applied to runtime: where the machine would pick
an arbitrary order, pick one.

| id | flags | fix |
|---|---|---|
| `determinism/no-hash-iteration` | iterating a `HashMap`/`HashSet` (`.iter()`, `for`, `.keys()`, `.values()`, `.collect()` into an ordered type) — use `BTreeMap`/`BTreeSet`, or sort explicitly | ◐ |
| `determinism/prefer-stable-sort` | `sort_unstable*` where equal elements are distinguishable | ● |
| `determinism/no-pointer-format` | `{:p}` and `as *const _ as usize` in output — addresses differ per run | ○ |
| `determinism/no-hash-in-api` | `HashMap`/`HashSet` in a public signature or serialised type, which exports the nondeterminism | ○ |

---

## `errors/*` — exceptions are `Result` and `?`

| id | flags | fix |
|---|---|---|
| `errors/no-unwrap` | `unwrap`, `expect`, `unwrap_unchecked` outside `tests/`, `#[cfg(test)]` and the world boundary | ◐ (`?`) |
| `errors/no-panic-macro` | `panic!`, `todo!`, `unimplemented!`, `unreachable!` in library code | ○ |
| `errors/no-index-panic` | `v[i]`, `s[a..b]` on slices/`Vec`/`String` → `.get(…)`, which returns the `Option` that was always there | ◐ |
| `errors/no-swallow` | `let _ = fallible()`, `.ok()` discarding an error, `unwrap_or_default()` on a `Result` — absence and failure conflated and then dropped | ○ |
| `errors/no-error-string-fmt` | building an error with `format!`, which turns a union of causes back into text | ○ |

---

## `structure/*` — the structure is the declaration

> *"There is no `canon.toml` and no lockfile, because the file tree already says
> everything they would."*

| id | flags | fix |
|---|---|---|
| `structure/no-mod-rs` | `mod.rs` — two spellings for one module, so pick `foo.rs` + `foo/` | ○ |
| `structure/no-inline-mod` | `mod foo { … }` with a body, except `#[cfg(test)] mod tests` | ○ |
| `structure/no-glob-import` | `use foo::*` — *"ambiguity is a hard error, not a shadowing rule"* | ◐ |
| `structure/no-import-alias` | `use foo::Bar as Baz` — a name the compiler cannot check against anything | ○ |
| `structure/no-path-attr` | `#[path = "…"]`, which unhooks the module tree from the file tree | ○ |
| `structure/no-formatter-config` | a `rustfmt.toml`/`.rustfmt.toml` that sets options. *The formatter has no options* | ● (delete) |
| `structure/no-reexport-facade` | `pub use` re-exports — a second path to one item | ○ |
| `structure/no-feature-branch` | `#[cfg(feature = …)]` in code, which is 2ⁿ programs in one tree | ○ |

---

## `comments/*` — documentation belongs in types and names

> *"If code needs explaining, the fix is a better type, not prose the compiler
> can't check and the next edit won't update."*

| id | flags | fix |
|---|---|---|
| `comments/no-comments` | any comment: `//`, `/* */`. In Canon these are lexer errors | ● (delete) |
| `comments/no-doc-comments` | `///`, `//!`, `#[doc]`. The Canon position, stated without softening | ○ |
| `comments/no-todo-marker` | `TODO`, `FIXME`, `HACK`, `XXX`, `WIP` — reported under its own id, because an unchecked promise deserves its own diagnostic even while the file still has comments in it | ○ |
| `comments/no-commented-code` | a comment whose body parses as Rust | ● (delete) |

---

## `meta/*` — there is no escape hatch

| id | flags | fix |
|---|---|---|
| `meta/no-suppression` | `#[allow(…)]`, `#[expect(…)]`, `#![allow(…)]`, `// rabot: ignore`. A per-line opt-out is a per-line style guide | ○ |
| `meta/no-dead-code-attr` | `#[allow(dead_code)]` specifically — dead code is deleted, not annotated | ● (delete the annotation, report the code) |

---

## `async/*` — async is a property of types, not syntax

> *"Concurrency is two combinators over the futures you already have — `Parallel`
> fans out, `Race` returns the winner and cancels the loser."*

| id | flags | fix |
|---|---|---|
| `async/no-runtime-in-library` | `#[tokio::main]`, `Runtime::new`, `block_on` outside the world boundary — the library picking an executor for its caller | ○ |
| `async/no-blocking-in-async` | `std::fs`/`std::net`/`thread::sleep`/`block_on` inside an `async fn` | ○ |
| `async/no-detached-spawn` | `spawn` whose handle is dropped → `join!` (Parallel) or `select!` (Race), so the concurrency is in the types | ○ |

---

## Considered and rejected

- **Banning traits.** Canon has none because constructor families replace them;
  Rust has no such replacement, so the rule would have nothing to point at.
- **1-based indexing.** Not reachable from a linter. Its cousin — one convention
  for ranges — survives as `form/prefer-inclusive-range`.
- **An abbreviation dictionary** (`ctx`, `req`, `cfg`, …). Endless, arbitrary, and
  the wrong half of the "names lie" argument — the fix is a type, not a longer
  word. `names/no-vague` covers the names that genuinely say nothing.
- **Sorting function parameters.** Position is meaning.
- **A `no-generics` rule.** Generics are Canon's input products, not a
  discretionary choice.
- **Severity levels and profiles.** One tier. See the top of this file.
