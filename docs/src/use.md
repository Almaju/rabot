# Use

```sh
rabot                      # = rabot check: lint the current directory, write nothing
rabot check --strict       # warnings fail the build too
rabot check --format json  # for editors and scripts

rabot fmt                  # sort fields, variants, impl items, derives,
                           # struct literals and patterns; then rustfmt
rabot fmt --check          # exit 1 if any file would change
rabot fmt --diff           # show what fmt would change, as a unified diff
rabot fmt --no-rustfmt     # reorder only, leave indentation alone

rabot check --changed      # only files with uncommitted changes
rabot fmt --changed=main   # only files touched since main

rabot hook                 # install a pre-commit hook
rabot rules                # every rule, its default level, the article behind it
rabot explain <rule>       # this documentation, in the terminal
rabot init                 # write a rabot.toml with every rule listed
```

## A diagnostic

```text
warning[primitive-soup]: `send_invoice` takes 3 `String` parameters (`user_id`, `email`, `invoice_id`): the compiler cannot tell them apart, so a swapped call site type-checks
  --> src/billing.rs:12:8
  = help: give each its own newtype (`struct UserId(String);`) and parse at the boundary
  = see: https://almaju.github.io/blog/docs/fundamentals/modeling/primitives
```

The first line names the rule and states the problem in terms of your code.
The help says what to write instead. The link is the argument.

## Exit codes

| Code | Meaning |
| --- | --- |
| 0 | clean, or warnings only without `--strict` |
| 1 | errors, or `fmt --check` found files to reorder |
| 2 | rabot itself failed: unreadable file, bad config |

## Migrate on contact

> Apply alphabetical ordering to all new code going forward. To any file
> you're already modifying. Don't create churn for its own sake.

An existing codebase does not need a big-bang reformat. `--changed` scopes
`check` and `fmt` to the files git sees as added, modified or untracked:
uncommitted work by default, or everything since a ref with
`--changed=<ref>`. The rest of the codebase is left alone until someone
touches it.

`rabot hook` installs exactly that as a git pre-commit hook: the commit is
refused (with the diff) when a staged file would be reordered, and the
domain rules run on the staged files.

## In CI

```yaml
- uses: almaju/rabot@main
```

installs the latest release, fails on anything `rabot fmt` would reorder
(printing the diff), then runs `rabot check --strict`. Inputs: `version`,
`args`, `fmt-check`, `working-directory`.
