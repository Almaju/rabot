# Configuration

`rabot init` writes a `rabot.toml` with every rule listed at its default
level. Everything has a default; the file may be empty or absent.

```toml
[rules]
# "allow" | "warn" | "error". Rules not listed keep their default.
free-function = "allow"
untyped-error = "error"

[thresholds]
oversized-impl = 20          # methods across inherent impls in one file
primitive-soup = 2           # parameters of the same primitive type
section-comments = 3         # leading comments in one function body
too-many-parameters = 7      # parameters, excluding self
vague-todo-min-words = 6     # words a TODO needs, unless it links a ticket

[naming]
vague-suffixes = ["Controller", "Coordinator", "Handler", "Helper", "Manager",
                  "Processor", "Repository", "Service", "UseCase", "Util", "Utils"]
orphan-modules = ["common", "helper", "helpers", "misc", "util", "utils"]
domain-fields = ["_id", "amount", "email", "latitude", "longitude", "password",
                 "phone", "price", "token", "url", "..."]
enum-fields = ["category", "kind", "level", "mode", "phase", "role", "stage",
               "state", "status"]
boundary-suffixes = ["Body", "Dto", "Params", "Payload", "Query", "Record",
                     "Request", "Response", "Row"]

[sorting]
# Pin derives to a position; the rest stay alphabetical in between.
derive-order = ["Debug", "Clone", "Copy", "...", "Serialize", "Deserialize"]

[tests]
# Rules that stay silent in test code.
relax = ["panic-in-production", "primitive-soup", "free-function", "..."]

[global-state]
allowed-names = ["LOG"]      # substring match, case-insensitive

[files]
exclude = ["target"]         # gitignore-style globs
```

Unknown keys are an error, so a typo cannot silently disable anything.

## Levels

| Level | Effect |
| --- | --- |
| `allow` | the rule never reports |
| `warn` | reported; exit code 0 unless `--strict` |
| `error` | reported; exit code 1 |

Defaults: every rule is `warn`, except `undocumented-exception`,
`unknown-rule` and `syntax-error`, which are `error`.
