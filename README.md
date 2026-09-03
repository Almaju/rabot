# rabot

A linter and formatter for Rust that enforces the principles of
[The Unwrap](https://almaju.github.io/blog/): sort everything, name what you
built, wrap your primitives, treat errors as data, and write down every
exception.

`rustfmt` decides where the whitespace goes. `clippy` catches bugs. rabot
enforces the opinions in between: the ones that decide whether a codebase
reads like architecture or like sediment.

## Install

Prebuilt binaries for Linux and macOS (x86_64 and arm64):

```sh
curl -fsSL https://raw.githubusercontent.com/almaju/rabot/main/install.sh | sh
```

Or with cargo, from source or from the release archives:

```sh
cargo install --git https://github.com/almaju/rabot --locked   # builds from source
cargo binstall --git https://github.com/almaju/rabot rabot      # downloads a release
```

Windows archives are attached to every
[release](https://github.com/almaju/rabot/releases). Each install provides
two binaries: `rabot`, and `cargo-rabot` so that `cargo rabot` works too.
rabot has no runtime dependencies; `rustfmt` is used when present.

## Use

```sh
rabot                  # = rabot check: lint the current directory, write nothing
rabot check --strict   # warnings fail the build too
rabot fmt              # sort fields, variants, impl items, derives, struct literals
                       # and patterns, then rustfmt the files it touched
rabot fmt --check      # exit 1 if any file would change
rabot fmt --diff       # show what fmt would change, as a unified diff
rabot check --changed  # only files with uncommitted changes
rabot fmt --changed=main   # only files touched since main
rabot hook             # install a pre-commit hook that does the two lines above
rabot rules            # every rule, its default level, the article behind it
rabot init             # write a rabot.toml with every rule listed
cargo rabot check      # same thing, as a cargo subcommand
```

`rabot fmt` reorders code and then runs `rustfmt` (with the edition of the
nearest `Cargo.toml`) on every file it rewrote, so the result is what
`cargo fmt` would produce. Pass `--no-rustfmt` to skip that step.

### In CI

```yaml
- uses: almaju/rabot@main
```

That installs the latest release, fails on anything `rabot fmt` would reorder
(showing the diff), then runs `rabot check --strict`. Inputs: `version` (a
release tag), `args` (default `check --strict`), `fmt-check` (default `true`)
and `working-directory`.

### Before every commit

`rabot hook` writes `.git/hooks/pre-commit`. The hook checks only the files
in the commit: `rabot fmt --check --changed`, then `rabot check --changed`.
`rabot hook --print` shows the script; `--force` replaces an existing hook.

## The rules

Every rule is one principle from one article. `fmt` fixes the first group;
`check` reports all of them.

### Sorting ([article](https://almaju.github.io/blog/docs/fundamentals/style/sorting))

> Sort your code alphabetically unless you have a documented reason not to.

| Rule | What is sorted |
| --- | --- |
| `sorted-fields` | Named struct fields, including struct-like enum variants |
| `sorted-variants` | Enum variants |
| `sorted-impl-items` | Inherent impls: consts, types, constructors, `pub` fns, private fns, each alphabetical. Trait impls: consts, types, fns |
| `sorted-trait-items` | Trait definitions: consts, types, fns |
| `sorted-struct-literal` | `User { .. }` initializers (fixed only when every initializer is plainly side-effect free) |
| `sorted-struct-pattern` | `let User { .. } = ..` and `match` patterns |
| `sorted-derives` | `#[derive(..)]` lists: alphabetical, except that a derive follows the trait it extends (`PartialEq, Eq`, `PartialOrd, Ord`, `Clone, Copy`) |

Order is case-insensitive and natural (`field2` before `field10`). Comments
before a member move with it; whitespace stays where it was, so single-line
lists stay single-line. `fmt` has been run over `clap_builder`, `toml` and
`ignore`; all three still compile afterwards.

rabot leaves alone lists whose order is semantic: `#[repr]` types, enums with
explicit discriminants, and enums that derive `PartialOrd` or `Ord`. Function
parameters are never sorted; calling convention is a real exception.

### Modeling

| Rule | Principle |
| --- | --- |
| `primitive-soup` | [Primitives](https://almaju.github.io/blog/docs/fundamentals/modeling/primitives): two parameters of the same primitive type can be swapped and still type-check. Wrap them. |
| `primitive-field` | [Primitives](https://almaju.github.io/blog/docs/fundamentals/modeling/primitives): `email: String`, `user_id: u64`, `latitude: f64`. The name promises a domain concept; the type accepts anything. Skipped for wire shapes (`*Request`, `*Row`, ...). |
| `free-function` | [Method Ownership](https://almaju.github.io/blog/docs/fundamentals/modeling/method-ownership): a free function whose primary parameter or return type is one of your types belongs on that type. |
| `vague-type-name` | [Method Ownership](https://almaju.github.io/blog/docs/fundamentals/modeling/method-ownership): `Service`, `Manager`, `Handler`, `Repository`... are names for decisions you have not made yet. |
| `orphan-module` | [Structs](https://almaju.github.io/blog/docs/fundamentals/modeling/structs): `utils`, `helpers`, `common` are where orphaned logic goes to die. |
| `oversized-impl` | [Structs](https://almaju.github.io/blog/docs/fundamentals/modeling/structs): more than 20 methods is several types that have not been separated yet. |
| `too-many-parameters` | [Structs](https://almaju.github.io/blog/docs/fundamentals/modeling/structs): parameters that travel together are a struct waiting to be named. |
| `stringly-typed-field` | [Primitives](https://almaju.github.io/blog/docs/fundamentals/modeling/primitives): `status: String`, `kind: String`, `role: String`. The valid values are an enum that has not been written yet. |
| `bypassable-constructor` | [Primitives](https://almaju.github.io/blog/docs/fundamentals/modeling/primitives): `pub struct Email(pub String)` with an `Email::parse` that validates. Anyone can write `Email(garbage)` and skip the door. |
| `swallowed-error` | [Errors](https://almaju.github.io/blog/docs/fundamentals/modeling/errors): an empty `Err(_) => {}` arm, an empty `if let Err(..)`, or `.ok();` as a statement. Every silent catch is a future 3am. |
| `panic-in-production` | [Errors](https://almaju.github.io/blog/docs/fundamentals/modeling/errors): `unwrap`, `expect`, `panic!`, `todo!` outside tests and `main` are bets that a call never fails. |
| `untyped-error` | [Errors](https://almaju.github.io/blog/docs/fundamentals/modeling/errors): `Box<dyn Error>`, `anyhow` and `Result<T, String>` erase the taxonomy callers need in order to decide. |

Signature rules (`primitive-soup`, `too-many-parameters`, `untyped-error`)
skip methods inside `impl Trait for T`: those signatures are the trait's
decision, not yours. They still apply to the trait definition itself.

### Architecture and style

| Rule | Principle |
| --- | --- |
| `global-state` | [Dependencies](https://almaju.github.io/blog/docs/fundamentals/architecture/dependencies): a `static` with interior mutability is a dependency hidden from every signature. A logger is fine. |
| `ignored-test` | [Tests](https://almaju.github.io/blog/docs/fundamentals/architecture/testing): `#[ignore]` without a reason. In six months nobody knows why three tests are skipped. `#[ignore = "why"]` is fine. |
| `mock-usage` | [Tests](https://almaju.github.io/blog/docs/fundamentals/architecture/testing): `mockall`, `faux`, `mock!`, `#[automock]`. Mocks test your assumptions; build the in-memory implementation. |
| `commented-out-code` | [Comments](https://almaju.github.io/blog/docs/fundamentals/style/comments): a comment that parses as Rust is code somebody could not delete. You have git. |
| `vague-todo` | [Comments](https://almaju.github.io/blog/docs/fundamentals/style/comments): `// TODO: refactor this` says nothing. Say what, why, or link the ticket. |
| `sectioned-function` | [Comments](https://almaju.github.io/blog/docs/fundamentals/style/comments): `// step 1`, `// step 2`, `// step 3` inside one body is a table of contents for functions that have not been extracted. Three or more section comments fire it. |

## Migrate on contact

> Apply alphabetical ordering to all new code going forward. To any file
> you're already modifying. Don't create churn for its own sake.

`--changed` scopes `check` and `fmt` to the files git sees as added,
modified or untracked: uncommitted work by default, or everything since a ref
with `--changed=<ref>`. `rabot hook` installs exactly that as a pre-commit
hook; a PR job can run `rabot fmt --check --changed=origin/main`. The rest of
the codebase is left alone until someone touches it.

## Test code is different

An `unwrap` in a test is the assertion. A `MockClock` under `#[cfg(test)]` is
exactly the injectable the testing article asks for. So inside test code
(`#[cfg(test)]` items, `#[cfg(any(test, ..))]`, `#[test]` functions, and
files under `tests/`, `benches/` or `examples/`) the domain rules stay
silent: `panic-in-production`, `swallowed-error`, `untyped-error`,
`primitive-soup`, `primitive-field`, `stringly-typed-field`,
`bypassable-constructor`, `free-function`, `vague-type-name`,
`orphan-module`, `oversized-impl`, `too-many-parameters`,
`sectioned-function`.

Sorting still applies, and so do the comment rules, `mock-usage` and
`ignored-test`: a test file is still code. The list is `[tests] relax` in `rabot.toml`; set it to
`[]` to hold tests to the full standard.

## Exceptions must be written down

> You can break the rule. You must document the exception.

```rust
// rabot: allow(sorted-fields) drop order matters: the guard must release first
struct Connection {
    guard: MutexGuard<'static, ()>,
    channel: Channel,
}
```

The comment silences the named rules for the item that follows it (or, as a
trailing comment, for its own line). `// rabot: allow-file(rule) reason`
covers the whole file.

The reason is not optional. An allow comment without one is reported as
`undocumented-exception`, at error level. A rule name rabot does not know is
`unknown-rule`.

## Configuration

`rabot init` writes a `rabot.toml` with every rule. Everything has a default;
the file may be empty or absent.

```toml
[rules]
free-function = "allow"        # "allow" | "warn" | "error"
untyped-error = "error"

[thresholds]
oversized-impl = 20
primitive-soup = 2
section-comments = 3
too-many-parameters = 7
vague-todo-min-words = 6

[naming]
boundary-suffixes = ["Body", "Dto", "Params", "Payload", "Query", "Record", "Request", "Response", "Row"]
domain-fields = ["_id", "amount", "email", "latitude", "longitude", "password", "phone",
                 "price", "token", "url", "..."]   # names or `_suffix`es that deserve a newtype
enum-fields = ["category", "kind", "level", "mode", "phase", "role", "stage", "state", "status"]
orphan-modules = ["common", "helper", "helpers", "misc", "util", "utils"]
vague-suffixes = ["Controller", "Coordinator", "Handler", "Helper", "Manager",
                  "Processor", "Repository", "Service", "UseCase", "Util", "Utils"]

[sorting]
# Pin derives to a position, as in cargo-sort-derives: names before "..."
# come first in this order, names after it come last, the rest stay
# alphabetical (with a derive after the trait it extends) in between.
derive-order = ["Debug", "Clone", "Copy", "...", "Serialize", "Deserialize"]

[tests]
# Rules that stay silent in test code (see "Test code is different").
relax = ["panic-in-production", "primitive-soup", "free-function", "..."]

[global-state]
allowed-names = ["LOG"]        # substring match, case-insensitive

[files]
exclude = ["target"]           # gitignore-style globs
```

## Exit codes

| Code | Meaning |
| --- | --- |
| 0 | Clean (or warnings only, without `--strict`) |
| 1 | Errors, or `fmt --check` found files to reorder |
| 2 | rabot itself failed (unreadable file, bad config) |

## Dogfooding

rabot is checked by rabot in CI: `rabot fmt --check src` and
`rabot check --strict src`. Its structs are alphabetical, its errors are
enums, and the two places it breaks its own rules carry a reason.
