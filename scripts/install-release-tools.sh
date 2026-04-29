#!/usr/bin/env sh
set -eu

INSTALL_DIR="${LIBAPRS_TOOL_INSTALL_DIR:-$HOME/.cargo/bin}"
TMP_ROOT="${TMPDIR:-/tmp}"

usage() {
  cat >&2 <<'USAGE'
Usage: scripts/install-release-tools.sh [tool...]

Installs pinned prebuilt release/security tools with SHA256 verification.
Supported tools: cargo-semver-checks, cargo-audit, cargo-deny

When no tools are listed, all supported tools are installed.
USAGE
}

verify_sha256() {
  expected="$1"
  archive="$2"

  if command -v sha256sum >/dev/null 2>&1; then
    printf '%s  %s\n' "$expected" "$archive" | sha256sum -c -
  elif command -v shasum >/dev/null 2>&1; then
    printf '%s  %s\n' "$expected" "$archive" | shasum -a 256 -c -
  else
    echo "no SHA256 verification tool found" >&2
    exit 1
  fi
}

install_release_tool() {
  name="$1"

  case "$name" in
    cargo-semver-checks)
      url="https://github.com/obi1kenobi/cargo-semver-checks/releases/download/v0.47.0/cargo-semver-checks-x86_64-unknown-linux-musl.tar.gz"
      sha256="daea6dfdebf9b15ce902a8af2fc6b9c2e86ddd49af17a9c5a656939289588f68"
      ;;
    cargo-audit)
      url="https://github.com/rustsec/rustsec/releases/download/cargo-audit/v0.22.1/cargo-audit-x86_64-unknown-linux-musl-v0.22.1.tgz"
      sha256="c32506f338bdcdaef5a17fb9f33abb6ecf9561324cfd34237fd335f9283a1eab"
      ;;
    cargo-deny)
      url="https://github.com/EmbarkStudios/cargo-deny/releases/download/0.19.4/cargo-deny-0.19.4-x86_64-unknown-linux-musl.tar.gz"
      sha256="3bd58b784e83715b86ddbc9deac591890372ec77fda5741bb0826970b958506f"
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unsupported release tool: $name" >&2
      usage
      exit 1
      ;;
  esac

  archive="$TMP_ROOT/${name}.tar.gz"
  extract_dir="$TMP_ROOT/${name}-extract"

  rm -rf "$extract_dir"
  mkdir -p "$INSTALL_DIR" "$extract_dir"

  curl -LsSf -o "$archive" "$url"
  verify_sha256 "$sha256" "$archive"
  tar -xzf "$archive" -C "$extract_dir"

  tool_path="$(find "$extract_dir" -type f -name "$name" | head -n 1 || true)"
  if [ -z "$tool_path" ]; then
    echo "could not find $name in downloaded archive" >&2
    exit 1
  fi

  cp "$tool_path" "$INSTALL_DIR/$name"
  chmod +x "$INSTALL_DIR/$name"
  "$INSTALL_DIR/$name" --version
}

if [ "$(uname -s)" != "Linux" ]; then
  case "${1:-}" in
    -h|--help)
      usage
      exit 0
      ;;
  esac

  echo "prebuilt release-tool installer currently supports Linux CI runners only" >&2
  exit 1
fi

if [ "$#" -eq 0 ]; then
  set -- cargo-semver-checks cargo-audit cargo-deny
fi

for tool in "$@"; do
  install_release_tool "$tool"
done
