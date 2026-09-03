# Install

## One line

Linux and macOS, x86_64 and arm64:

```sh
curl -fsSL https://raw.githubusercontent.com/almaju/rabot/main/install.sh | sh
```

The script downloads the archive for your platform from the latest
[release](https://github.com/almaju/rabot/releases), verifies its checksum,
and installs `rabot` and `cargo-rabot` into `~/.cargo/bin` (or `~/.local/bin`
when there is no cargo). Two variables tune it:

| Variable | Meaning |
| --- | --- |
| `RABOT_VERSION` | a release tag, e.g. `v0.1.2` (default: latest) |
| `RABOT_INSTALL` | the directory to install into |

## With cargo

```sh
cargo install --git https://github.com/almaju/rabot --locked   # builds from source
cargo binstall --git https://github.com/almaju/rabot rabot      # downloads a release
```

## Windows

Every release carries a `x86_64-pc-windows-msvc.zip`. Unpack it and put the
two executables on your `PATH`.

## What you get

Two binaries, `rabot` and `cargo-rabot`, so both `rabot check` and
`cargo rabot check` work. There are no runtime dependencies. `rustfmt` is used
when present, to re-indent after `rabot fmt`.
