# rabot

A linter and formatter for Rust that enforces the principles of
[The Unwrap](https://almaju.github.io/blog/): sort everything, name what you
built, wrap your primitives, treat errors as data, and write down every
exception.

`rustfmt` decides where the whitespace goes. `clippy` catches bugs. rabot
enforces the opinions in between: the ones that decide whether a codebase
reads like architecture or like sediment.

## Install

```sh
cargo install --git https://github.com/almaju/rabot
```

## Use

```sh
rabot check            # lint the current directory, write nothing
rabot check --strict   # warnings fail the build too
rabot fmt              # sort fields, variants, impl items, derives, struct literals
rabot fmt --check      # exit 1 if any file would change
rabot fmt --diff       # show what fmt would change, as a unified diff
rabot rules            # every rule, its default level, the article behind it
rabot init             # write a rabot.toml with every rule listed
```

rabot reorders code but does not re-indent it. Run `cargo fmt` after
`rabot fmt`.

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
| `sorted-derives` | `#[derive(..)]` lists |

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
| `free-function` | [Method Ownership](https://almaju.github.io/blog/docs/fundamentals/modeling/method-ownership): a free function whose primary parameter or return type is one of your types belongs on that type. |
| `vague-type-name` | [Method Ownership](https://almaju.github.io/blog/docs/fundamentals/modeling/method-ownership): `Service`, `Manager`, `Handler`, `Repository`... are names for decisions you have not made yet. |
| `orphan-module` | [Structs](https://almaju.github.io/blog/docs/fundamentals/modeling/structs): `utils`, `helpers`, `common` are where orphaned logic goes to die. |
| `oversized-impl` | [Structs](https://almaju.github.io/blog/docs/fundamentals/modeling/structs): more than 20 methods is several types that have not been separated yet. |
| `too-many-parameters` | [Structs](https://almaju.github.io/blog/docs/fundamentals/modeling/structs): parameters that travel together are a struct waiting to be named. |
| `panic-in-production` | [Errors](https://almaju.github.io/blog/docs/fundamentals/modeling/errors): `unwrap`, `expect`, `panic!`, `todo!` outside tests and `main` are bets that a call never fails. |
| `untyped-error` | [Errors](https://almaju.github.io/blog/docs/fundamentals/modeling/errors): `Box<dyn Error>` and `anyhow` erase the taxonomy callers need in order to decide. |

Signature rules (`primitive-soup`, `too-many-parameters`, `untyped-error`)
skip methods inside `impl Trait for T`: those signatures are the trait's
decision, not yours. They still apply to the trait definition itself.

### Architecture and style

| Rule | Principle |
| --- | --- |
| `global-state` | [Dependencies](https://almaju.github.io/blog/docs/fundamentals/architecture/dependencies): a `static` with interior mutability is a dependency hidden from every signature. A logger is fine. |
| `mock-usage` | [Tests](https://almaju.github.io/blog/docs/fundamentals/architecture/testing): `mockall`, `faux`, `mock!`, `#[automock]`. Mocks test your assumptions; build the in-memory implementation. |
| `commented-out-code` | [Comments](https://almaju.github.io/blog/docs/fundamentals/style/comments): a comment that parses as Rust is code somebody could not delete. You have git. |
| `vague-todo` | [Comments](https://almaju.github.io/blog/docs/fundamentals/style/comments): `// TODO: refactor this` says nothing. Say what, why, or link the ticket. |

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
too-many-parameters = 7
vague-todo-min-words = 6

[naming]
orphan-modules = ["common", "helper", "helpers", "misc", "util", "utils"]
vague-suffixes = ["Controller", "Coordinator", "Handler", "Helper", "Manager",
                  "Processor", "Repository", "Service", "UseCase", "Util", "Utils"]

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
