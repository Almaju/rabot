# Say what is deliberate

> You can break the rule. You must document the exception.

An exception comment says *that* the code is deliberate. Say *what* is
deliberate, between the marker and the colon, and the comment covers that and
nothing else:

```rust
// Deliberately unsorted: the guard must release before the pool.
struct Connection {
    guard: MutexGuard<'static, ()>,
    pool: Pool,
    email: String,
}
```

*Unsorted* names the sorting rules, so the fields keep their order. The
`email: String` is still reported as a
[primitive field](rules/primitive-field.md): the comment never said that part
was deliberate.

The words are the ones you would use anyway:

```rust
let first = items.first().unwrap(); // Intentional unwrap: checked non-empty two lines up.

// Intentionally a String: a free-form user label, not a closed set.
struct Tag { label: String }

// Deliberately long: mirrors the C ABI of libfoo_render exactly.
fn render(..) { .. }
```

## How the words are read

- **Where**: from the marker to the first colon, comma or full stop. The
  reason after it is never searched: `// Intentional: the unwrap is checked
  above` names nothing.
- **Which**: the words in the table below, in any case, singular or plural
  (*unwrap*, *Unwraps*, *`String`*). Other words around them are fine:
  *Deliberately left unsorted*, *Intentionally a helpers module*.
- **Several**: each word adds its rules. *Deliberately unsorted and
  unwrapped* covers the sorting rules and `panic-in-production`. A word can
  name more than one rule: *long* fits a long parameter list, a long impl
  and a long function alike.
- **None**: a comment that names no rule covers every rule on the item.
  *Intentional: ...*, *By design, ...* and *Intentionally slow: ...* all do.
  Nothing is reported; the comment is simply broad.

Leave the words out when the exception really is about the whole item, or
when one line breaks two rules for one reason:

```rust
let port = env::var("PORT").unwrap(); // Intentional: the deploy script validates PORT.
```

## The words

| Rule | Words |
| --- | --- |
| [sorted-fields](rules/sorted-fields.md), [sorted-variants](rules/sorted-variants.md), [sorted-impl-items](rules/sorted-impl-items.md), [sorted-trait-items](rules/sorted-trait-items.md), [sorted-struct-literal](rules/sorted-struct-literal.md), [sorted-struct-pattern](rules/sorted-struct-pattern.md), [sorted-derives](rules/sorted-derives.md) | *unsorted*, *sorted*, *sort*, *sorting*, *order*, *ordered*, *ordering*, *unordered* |
| [primitive-soup](rules/primitive-soup.md), [primitive-field](rules/primitive-field.md) | *plain*, *primitive*, *raw* |
| [stringly-typed-field](rules/stringly-typed-field.md) | *string*, *str*, *stringly* |
| [bypassable-constructor](rules/bypassable-constructor.md) | *public*, *pub*, *exposed* |
| [boolean-validation](rules/boolean-validation.md) | *bool*, *boolean* |
| [free-function](rules/free-function.md) | *free* |
| [vague-type-name](rules/vague-type-name.md) | *named*, *name* |
| [orphan-module](rules/orphan-module.md) | *module*, *helper*, *util* |
| [oversized-impl](rules/oversized-impl.md) | *large*, *big*, *long*, *oversized* |
| [too-many-parameters](rules/too-many-parameters.md) | *long*, *parameter*, *param*, *argument*, *arg* |
| [panic-in-production](rules/panic-in-production.md) | *unwrap*, *unwrapped*, *expect*, *panic*, *panicking* |
| [untyped-error](rules/untyped-error.md) | *untyped*, *boxed*, *dyn*, *anyhow* |
| [swallowed-error](rules/swallowed-error.md) | *ignored*, *swallowed*, *discarded*, *best-effort* |
| [dropped-error-context](rules/dropped-error-context.md) | *context* |
| [escape-hatch-variant](rules/escape-hatch-variant.md) | *catch-all*, *free-form*, *escape*, *hatch* |
| [global-state](rules/global-state.md) | *global*, *static*, *singleton* |
| [ambient-config](rules/ambient-config.md) | *env*, *environment*, *config*, *configuration* |
| [module-cycle](rules/module-cycle.md) | *circular*, *cycle*, *cyclic* |
| [mock-usage](rules/mock-usage.md) | *mocked*, *mock* |
| [ignored-test](rules/ignored-test.md) | *ignored*, *skipped* |
| [ambient-time](rules/ambient-time.md) | *clock*, *time*, *timing* |
| [ambient-randomness](rules/ambient-randomness.md) | *random*, *randomness*, *rng*, *jitter* |
| [sleep-in-tests](rules/sleep-in-tests.md) | *sleep*, *sleeping* |
| [commented-out-code](rules/commented-out-code.md) | *kept*, *commented*, *commented-out* |
| [sectioned-function](rules/sectioned-function.md) | *step*, *section*, *sectioned*, *long* |
| [vague-todo](rules/vague-todo.md) | none: write the sentence the TODO is missing |

[undocumented-exception](rules/undocumented-exception.md),
[unknown-rule](rules/unknown-rule.md) and
[syntax-error](rules/syntax-error.md) have no words: they are about the
exception itself, and no exception silences them.

## When a word is not enough

To name rules by their ids, or to cover a whole file, use
[`rabot: allow`](exceptions.md#naming-the-rule).
