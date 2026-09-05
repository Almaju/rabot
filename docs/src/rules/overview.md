# Rules

Every rule is one principle from one article. `fmt` fixes the sorting
family; `check` reports all of them. Levels are the defaults; every one can
be changed in `rabot.toml`.

| Rule | Level | Fix | Principle |
| --- | --- | --- | --- |
| [sorted-fields](sorted-fields.md) | warn | fmt | Sort everything |
| [sorted-variants](sorted-variants.md) | warn | fmt | Sort everything |
| [sorted-impl-items](sorted-impl-items.md) | warn | fmt | Sort everything |
| [sorted-trait-items](sorted-trait-items.md) | warn | fmt | Sort everything |
| [sorted-struct-literal](sorted-struct-literal.md) | warn | fmt | Sort everything |
| [sorted-struct-pattern](sorted-struct-pattern.md) | warn | fmt | Sort everything |
| [sorted-derives](sorted-derives.md) | warn | fmt | Sort everything |
| [primitive-soup](primitive-soup.md) | warn | | Wrap your primitives |
| [primitive-field](primitive-field.md) | warn | | Wrap your primitives |
| [stringly-typed-field](stringly-typed-field.md) | warn | | Wrap your primitives |
| [bypassable-constructor](bypassable-constructor.md) | warn | | Validate once, at construction |
| [boolean-validation](boolean-validation.md) | warn | | Treat errors as data |
| [free-function](free-function.md) | warn | | Put behavior on the type it belongs to |
| [vague-type-name](vague-type-name.md) | warn | | Name what you built |
| [orphan-module](orphan-module.md) | warn | | Name what you built |
| [oversized-impl](oversized-impl.md) | warn | | Obsess over your data structures |
| [too-many-parameters](too-many-parameters.md) | warn | | Obsess over your data structures |
| [panic-in-production](panic-in-production.md) | warn | | Treat errors as data |
| [untyped-error](untyped-error.md) | warn | | Treat errors as data |
| [swallowed-error](swallowed-error.md) | warn | | Treat errors as data |
| [dropped-error-context](dropped-error-context.md) | warn | | Treat errors as data |
| [escape-hatch-variant](escape-hatch-variant.md) | warn | | Treat errors as data |
| [global-state](global-state.md) | warn | | Dependencies in the signature |
| [ambient-config](ambient-config.md) | warn | | Dependencies in the signature |
| [mock-usage](mock-usage.md) | warn | | Real implementations, not mocks |
| [ignored-test](ignored-test.md) | warn | | Write down every exception |
| [ambient-time](ambient-time.md) | warn | | Make the clock injectable |
| [ambient-randomness](ambient-randomness.md) | warn | | Make the clock injectable |
| [sleep-in-tests](sleep-in-tests.md) | warn | | Fast and honest tests |
| [commented-out-code](commented-out-code.md) | warn | | Delete the comment, fix the code |
| [vague-todo](vague-todo.md) | warn | | Delete the comment, fix the code |
| [sectioned-function](sectioned-function.md) | warn | | Delete the comment, fix the code |
| [undocumented-exception](undocumented-exception.md) | error | | Write down every exception |
| [unknown-rule](unknown-rule.md) | error | | Write down every exception |
| [syntax-error](syntax-error.md) | error | | |

`rabot rules` prints the same table in the terminal; `rabot explain <rule>`
prints a rule's page.
