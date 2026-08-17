# rabot — a Canon-inspired linter for Rust

`rabot` is a linter that applies [Canon](https://almaju.github.io/canon/)'s
philosophy to Rust:

> Wherever a choice is discretionary, the compiler removes the choice or
> enforces one answer.

Canon can delete `if`, `let`, comments and imports because it owns its grammar.
Rust's grammar is fixed, so `rabot` does the only thing left: it **refuses the
discretionary spellings** Rust allows, and **fixes mechanically** the ones that
have exactly one canonical answer. The target is Canon's real payoff — *two
programmers writing the same program produce the same bytes, and a diff only
ever shows meaning.*

The rule catalogue lives in [RULES.md](./RULES.md). This document is the *how*.

---

## 1. What rabot is, and is not

| rabot is | rabot is not |
|---|---|
| A canonical-form checker with a real auto-fixer | A formatter — `rustfmt` already is one, and it has the options we refuse to add |
| Opinionated to the point of being un-idiomatic (at the top tier) | A clippy replacement — clippy finds bugs, rabot removes choices |
| Zero-configuration | A framework with a plugin API and per-rule severities |
| Syntax-first, so it runs on stable Rust in milliseconds | A type checker — the type-aware rules are explicitly phase 6 |

Where a rule overlaps clippy (`unwrap_used`, `wildcard_imports`, …) rabot still
implements it natively: the point is that one binary with no configuration
produces the verdict. RULES.md names the clippy equivalent for every overlap so
a team can choose.

---

## 2. Decisions to make before code

These four shape the architecture. My recommendation is marked **✔**.

### 2.1 Front end

| Option | Gets us | Costs |
|---|---|---|
| **✔ `syn` 2 + our own trivia lexer** | Whole ordering/form/naming/structure catalogue, stable toolchain, sub-second runs, trivial distribution (`cargo install rabot`) | No types: exhaustiveness, purity and "is this scrutinee an enum" are heuristics |
| `ra_ap_syntax` (rust-analyzer's rowan CST) | Lossless tree — comments and whitespace are nodes, error-tolerant parsing, fixes are tree edits | Unstable published API, version churn every release, heavy dep, still no types without `ra_ap_hir` |
| `rustc_private` driver / `dylint` | Real HIR + types: true exhaustiveness, real purity analysis | Nightly pin, per-toolchain rebuilds, slow, hard to install, hostile to a "one binary, no config" story |

Recommendation: start with `syn`, and design the engine so a second, type-aware
back end can add *precision* to existing rule IDs later (phase 6) without
changing the rules' names or their diagnostics. A rule declares what it needs
(`Needs::Syntax` / `Needs::Types`); type-needing rules simply don't exist until
that back end does.

`syn` discards comments and whitespace, which the `comments/*` rules and the
ordering fixer both need, so we pair it with a small **trivia lexer** (~300
lines: nested block comments, raw strings, byte/char literals, shebang) that
produces the byte ranges of every comment and every blank-line run. That lexer
is also what lets the fixer move an item *together with* its attributes and doc
comments.

### 2.2 Configuration: none

Canon's formatter has no options, so neither does rabot. No `rabot.toml`, no
per-rule severity, no `#[allow]`.

Tiers still have to be selectable, and Canon's own answer to "config files
restate what the tree already says" is a marker in the tree. So: the tier is
named by an empty marker file at the repo root.

```
.rabot/strict        # absent → core tier
.rabot/canonical
```

`rabot use strict` creates the marker; `rabot use` prints the active tier and
why. One word, versioned in git, nothing to argue about.

Escape hatches are deliberately absent: `#[allow(...)]`, `#[expect(...)]` and
`// rabot: ignore` are themselves violations (`meta/no-suppression`). The only
exits are the tier and `--only`/`--except` on the command line for triage, which
CI does not pass.

### 2.3 How far into anti-idiomatic Rust

Three tiers, each a superset. A rule's tier is part of its identity, not a
setting:

- **core** — every rule whose fix has exactly one answer and whose result a
  normal Rust team would accept: all of `order/*`, `determinism/*`,
  `structure/*`, `errors/*`, `caps/*`. This is the tier we recommend and the
  one CI examples use.
- **strict** — adds the rules that fight convention but not the language:
  `types/newtype-primitives`, `names/single-use-let`, `comments/no-comments`
  (doc comments survive), `form/no-loop-keywords`.
- **canonical** — the full doctrine: no `if` at all, no untyped `let`, no doc
  comments, one type per file, inclusive ranges, evidence-returning effects.
  Explicitly a statement, not a recommendation.

### 2.4 Dogfooding

rabot's own source passes **strict** in CI, and every violation of `canonical`
in its own tree is listed in `docs/DOGFOOD.md` with a reason. A linter that
can't survive its own top tier should say so out loud rather than ship the tier.

---

## 3. Architecture

```
rabot/
├── crates/
│   ├── rabot-core/     # source map, trivia lexer, diagnostics, fix engine, rule registry
│   ├── rabot-rules/    # one module per group; the catalogue in RULES.md
│   └── rabot-cli/      # the `rabot` binary
├── tests/fixtures/<rule-id>/{ok.rs, bad.rs, bad.stderr, bad.fixed.rs}
└── docs/{PLAN.md, RULES.md, rules/<rule-id>.md}
```

Three crates, not one, so the rule catalogue can't reach into CLI concerns and
the fix engine stays testable without rules.

### 3.1 Core types

```rust
pub struct RuleMeta {
    pub id: &'static str,          // "order/enum-variants"
    pub tier: Tier,                // Core | Strict | Canonical
    pub needs: Needs,              // Syntax | Types
    pub fixable: Fixable,          // Always | Sometimes | Never
    pub canon: &'static str,       // link into the Canon philosophy section it comes from
    pub doc: &'static str,         // rendered by `rabot explain`
}

pub trait Rule: Sync {
    fn meta(&self) -> &'static RuleMeta;
    fn check(&self, file: &SourceFile, out: &mut Report);
}

pub struct SourceFile {
    pub path: PathBuf,
    pub text: String,
    pub ast: syn::File,
    pub trivia: Trivia,     // comments + blank-line runs, by byte range
    pub lines: LineIndex,   // byte offset ⇄ line/col
}
```

A `Report` collects `Diagnostic { rule, span, message, notes, fix: Option<Fix> }`
where `Fix { label, edits: Vec<Edit { range: Range<usize>, text: String }> }`.
Everything is byte ranges over the original text — no rule ever produces a
string of Rust by pretty-printing the whole AST, because that would reformat
untouched code and destroy comments.

### 3.2 The fix engine

`--fix` is the feature that makes the ordering rules worth anything, so it gets
the strictest contract:

1. Run all rules, collect every `Fix`.
2. Sort edits by start offset; drop any edit overlapping an already-accepted one
   (deterministically: lower rule id wins), so one pass is always coherent.
3. Apply the surviving edits back-to-front.
4. **Verify**: re-parse the result with `syn`. If it fails, discard the whole
   file's edits and report an internal error with the rule ids involved. A fixer
   never writes unparseable code.
5. Loop from 1 until no fixes remain, capped at 8 passes.
6. Run `rustfmt` on changed files (Canon: layout is not ours to choose), unless
   `--no-format`.

`--fix` refuses to run on a dirty git worktree unless `--allow-dirty`, the same
bargain `cargo fix` makes.

### 3.3 Moving items safely (the ordering fixer)

Sorting an item means moving its *whole* text: leading `#[attr]`s, its
`///`/`//!` doc comments, and any comment lines glued to it above with no blank
line between. The extended range runs from the start of that leading block to
the item's last byte; blank-line runs *between* items stay where they are, so a
sort never collapses or invents blank lines. Attached-trailing comments
(`}` followed by `// note` on the same line) travel with the item too.

Items exempt from ordering, because Canon exempts the equivalent:

- `fn main` and any `#[…::main]`-attributed entry — a distinguished role.
- `#[cfg(test)] mod tests` — pinned last.
- Anything inside a macro invocation body (we cannot see it; we say so).

### 3.4 Sort key

Byte-wise, case-sensitive comparison of the UTF-8 name, exactly as Canon
specifies: digits < uppercase < lowercase, so `notFound` precedes
`noteOneBody`. No locale, no `_`-stripping, no "smart" numeric sort — the whole
point is that nobody has to compute it, `rabot check --fix` does.

Top-level items are grouped by kind before sorting, because Rust has kinds Canon
doesn't: `use` → `mod` → `const`/`static` → `type` → `struct`/`enum`/`union` →
`trait` → `impl` → `fn`. Within a group: alphabetical. `impl` blocks key on
`(trait_path, self_ty)` with inherent impls before trait impls.

### 3.5 Output

- Default: rustc-style annotated snippets (`annotate-snippets`), one per
  diagnostic, with the rule id and a `help:` line naming the fix.
- `--format json`: stable schema for editors and CI (`{rule, path, span, message, fix}`).
- `--format github`: `::error file=…,line=…::` workflow commands.
- Exit codes: `0` clean, `1` violations, `2` internal error (parse failure,
  fixer verification failure).

### 3.6 CLI

Mirrors Canon's one-binary surface:

```sh
rabot check [PATHS]        # default: the whole workspace, respecting .gitignore
rabot check --fix          # fix what is mechanical, then re-check
rabot check --diff         # print the fix as a patch, change nothing
rabot explain order/enum-variants
rabot list [--tier strict] # the catalogue, machine-readable with --format json
rabot use [strict|canonical]
```

`rabot check` with no paths walks `**/*.rs` via the `ignore` crate, skipping
`target/` and anything gitignored. Files are parsed and linted in parallel with
`rayon`; the report is sorted by path then offset so output is identical run to
run (that being the whole thesis).

### 3.7 Dependencies

`syn` (full, extra-traits, visit), `proc-macro2` (span-locations), `annotate-snippets`,
`clap`, `ignore`, `rayon`, `serde`/`serde_json`, `anyhow`. Dev: `insta` for
snapshot fixtures. Nothing else — no async runtime, no config crate, no
`lazy_static` (our own `caps/no-globals` forbids it).

---

## 4. Testing

Fixture-driven, one directory per rule id:

```
tests/fixtures/order/enum-variants/
├── ok.rs           # must produce zero diagnostics
├── bad.rs          # must produce exactly the snapshot
├── bad.stderr      # the rendered diagnostic (insta snapshot, `--bless`-able)
└── bad.fixed.rs    # the result of --fix, which must itself be `ok`
```

Three invariants the harness enforces for every rule:

1. **Idempotence** — fixing `bad.fixed.rs` changes nothing.
2. **Convergence** — `bad.rs` fixed once is clean; no rule needs two passes to
   settle its own output.
3. **Parse safety** — every `*.fixed.rs` parses, and (for a subset marked
   `compile`) `cargo check`s.

Plus a corpus run: `rabot check` over a handful of vendored real crates, asserted
only for "no panics, no unparseable fixes", to catch fixer edge cases that
hand-written fixtures miss.

---

## 5. Milestones

| # | Deliverable | Why here |
|---|---|---|
| **M0** | Workspace, `SourceFile`, trivia lexer, diagnostics rendering, fix engine, fixture harness, and exactly one rule end-to-end: `order/enum-variants` with `--fix` | Proves the hard part (safe text edits with comments attached) before any breadth |
| **M1** | All of `order/*` + `rabot explain` + `--format json` | The differentiator; nothing else on the market sorts *and* fixes Rust declarations |
| **M2** | `form/*` and `dispatch/*` — including `form/no-if` desugaring to `match` with `false`/`true` arms in that order | The second-most visible Canon idea: branching is dispatch |
| **M3** | `caps/*` + `determinism/*` | The rules with the highest real-world value: no ambient authority, no HashMap order leaking into output |
| **M4** | `names/*`, `types/*`, `structure/*`, `comments/*`, `meta/*`, tiers + `.rabot/` marker | Completes the catalogue |
| **M5** | Distribution: `cargo install`, prebuilt binaries, GitHub Action, pre-commit hook, `--format github` | Adoption |
| **M6** | Optional type-aware back end (`Needs::Types`) behind a feature: real exhaustiveness, real purity, precise `caps/*` | Precision, once the shape is settled |

M0–M2 are the ones worth committing to now; M3+ should be re-planned once the
fixer has met real code.

---

## 6. Known hard problems

- **Enum vs. not.** `dispatch/no-wildcard-arm` needs to know the scrutinee is an
  enum. Without types we heuristically treat a match whose arms are path
  patterns (`Foo::Bar`, `Some(_)`) as a union dispatch and stay silent
  otherwise. This under-reports on purpose; false negatives are survivable,
  false positives are not.
- **Macros.** `syn` gives us token streams, not items, inside macro
  invocations. Rules skip macro bodies and `rabot check` prints a one-line
  summary of how many item positions it could not see, so the silence is
  visible.
- **`order/struct-literal-fields`.** Reordering field initialisers changes
  evaluation order, and Canon never reorders operands. So the rule only fixes
  when every value is a path, literal or field access; otherwise it reports
  without a fix.
- **`form/no-if` fidelity.** `if` as an expression, `else if` chains, `if let`
  with guards, and `?`/`return` inside branches all desugar to `match`, but the
  fix must preserve `mut`/ref binding modes. The fixer handles the shapes it can
  prove and reports (no fix) on the rest rather than guessing.
- **Overlapping fixes.** `order/items` and `comments/no-commented-code` can
  target the same bytes. The overlap-drop rule in §3.2 keeps a pass coherent,
  and the loop lets the loser land on the next pass.

---

## 7. Not doing, deliberately

- **Reordering call arguments, tuple fields, or list elements.** Position is
  meaning; Canon says so explicitly and so do we.
- **Reimplementing formatting.** `rustfmt` is the canonical layout. rabot only
  refuses a `rustfmt.toml` that sets options (`structure/no-formatter-config`).
- **Banning traits.** Canon has no traits because it has constructor families;
  Rust traits are load-bearing and there is no Canon-faithful replacement to
  point at.
- **1-based indexing.** Unreachable from a linter. Its cousin — one convention
  for ranges — survives as `form/prefer-inclusive-range` at the canonical tier.
- **A plugin API.** Every rule ships in the binary. Extensibility is another
  word for configuration.
