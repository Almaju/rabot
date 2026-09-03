# rabot

A linter and formatter for Rust that enforces the principles of
[The Unwrap](https://almaju.github.io/blog/): sort everything, name what you
built, wrap your primitives, treat errors as data, and write down every
exception.

`rustfmt` decides where the whitespace goes. `clippy` catches bugs. rabot
enforces the opinions in between: the ones that decide whether a codebase
reads like architecture or like sediment.

```sh
curl -fsSL https://raw.githubusercontent.com/almaju/rabot/main/install.sh | sh
rabot            # lint the current directory
rabot fmt        # sort what can be sorted, then rustfmt the files it touched
```

## How to read the rules

Every rule page has the same shape, so you can skim to the part you need:

- **Principle**: the one sentence from the article the rule enforces.
- **What it checks**: exactly when it fires.
- **Don't** / **Do**: the code it rejects, and what to write instead.
- **Silence it**: the allow comment or the config key, when you have a
  reason. The reason is not optional.

Each page links to the article that makes the full argument. The rule is the
enforcement; the article is the why.

## Two commands

`rabot check` reports every principle violation and writes nothing.
`rabot fmt` rewrites what a machine can fix safely: order. Fields, variants,
impl items, derives, struct literals and patterns. Nothing else changes, and
`rustfmt` runs afterwards so the result is what `cargo fmt` would produce.

rabot is checked by rabot. Its own source passes `rabot fmt --check` and
`rabot check --strict` in CI, and the handful of places where it breaks its
own rules carry a written reason.
