#!/bin/sh
# Install rabot from the latest GitHub release.
#
#   curl -fsSL https://raw.githubusercontent.com/almaju/rabot/main/install.sh | sh
#
# Options (environment variables):
#   RABOT_VERSION   tag to install, e.g. v0.1.0 (default: latest release)
#   RABOT_INSTALL   directory to install into (default: ~/.cargo/bin if it
#                   exists, otherwise ~/.local/bin)
set -eu

repo="almaju/rabot"

fail() {
    echo "install.sh: $*" >&2
    exit 1
}

need() {
    command -v "$1" >/dev/null 2>&1 || fail "$1 is required"
}

need curl
need tar

os=$(uname -s)
arch=$(uname -m)
case "$os" in
    Linux) os_part="unknown-linux-gnu" ;;
    Darwin) os_part="apple-darwin" ;;
    *) fail "unsupported OS: $os (use: cargo install --git https://github.com/$repo --locked)" ;;
esac
case "$arch" in
    x86_64 | amd64) arch_part="x86_64" ;;
    arm64 | aarch64) arch_part="aarch64" ;;
    *) fail "unsupported architecture: $arch (use: cargo install --git https://github.com/$repo --locked)" ;;
esac
target="$arch_part-$os_part"

version="${RABOT_VERSION:-}"
if [ -z "$version" ]; then
    version=$(curl -fsSL "https://api.github.com/repos/$repo/releases/latest" \
        | sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p' | head -n 1)
    [ -n "$version" ] || fail "could not determine the latest release"
fi

name="rabot-$version-$target"
url="https://github.com/$repo/releases/download/$version/$name.tar.gz"

install_dir="${RABOT_INSTALL:-}"
if [ -z "$install_dir" ]; then
    if [ -d "$HOME/.cargo/bin" ]; then
        install_dir="$HOME/.cargo/bin"
    else
        install_dir="$HOME/.local/bin"
    fi
fi
mkdir -p "$install_dir"

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

echo "downloading $url"
curl -fsSL "$url" -o "$tmp/$name.tar.gz"
if curl -fsSL "$url.sha256" -o "$tmp/$name.tar.gz.sha256" 2>/dev/null; then
    expected=$(cut -d' ' -f1 "$tmp/$name.tar.gz.sha256")
    if command -v shasum >/dev/null 2>&1; then
        actual=$(shasum -a 256 "$tmp/$name.tar.gz" | cut -d' ' -f1)
    else
        actual=$(sha256sum "$tmp/$name.tar.gz" | cut -d' ' -f1)
    fi
    [ "$expected" = "$actual" ] || fail "checksum mismatch for $name.tar.gz"
fi
tar xzf "$tmp/$name.tar.gz" -C "$tmp"
install -m 755 "$tmp/$name/rabot" "$tmp/$name/cargo-rabot" "$install_dir/"

echo "installed rabot $version to $install_dir"
case ":$PATH:" in
    *":$install_dir:"*) ;;
    *) echo "add $install_dir to your PATH" ;;
esac
"$install_dir/rabot" --version
