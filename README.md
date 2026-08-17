# rabot

A maximally opinionated linter for Rust, inspired by
[Canon](https://almaju.github.io/canon/):

> Wherever a choice is discretionary, the compiler removes the choice or
> enforces one answer.

Rust's grammar is fixed, so `rabot` does the only thing left: it refuses the
discretionary spellings Rust allows, and mechanically fixes the ones with exactly
one canonical answer. Declarations sort alphabetically. Branching is a `match`
with no wildcard. Effects arrive as parameters, never from ambient authority.
There is no configuration file, because there is nothing to have an opinion
about.

```sh
rabot check              # canonical form, ordering, capabilities
rabot check --fix        # fix what is mechanical, then re-check
rabot explain order/enum-variants
```

**Status: planning.** Nothing is implemented yet.

- [docs/PLAN.md](docs/PLAN.md) — architecture, open decisions, milestones
- [docs/RULES.md](docs/RULES.md) — the rule catalogue and its Canon provenance

## License

MIT
