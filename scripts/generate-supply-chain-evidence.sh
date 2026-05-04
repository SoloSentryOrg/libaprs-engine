#!/usr/bin/env sh
set -eu

ROOT_DIR="$(
  CDPATH=
  cd -- "$(dirname -- "$0")/.." && pwd
)"

usage() {
  cat >&2 <<'USAGE'
Usage: scripts/generate-supply-chain-evidence.sh [--check]

Generates repository supply-chain evidence:
  supply-chain/sbom/*.cdx.json
  supply-chain/SHA256SUMS

Use --check to regenerate evidence in a temporary directory and compare it with
the tracked files.
USAGE
}

fail() {
  echo "generate-supply-chain-evidence: $*" >&2
  exit 1
}

has_tool() {
  command -v "$1" >/dev/null 2>&1
}

sha256_file() {
  file="$1"

  if has_tool sha256sum; then
    sha256sum "$file" | awk '{ print $1 }'
  elif has_tool shasum; then
    shasum -a 256 "$file" | awk '{ print $1 }'
  else
    fail "no SHA-256 tool found; install sha256sum or shasum"
  fi
}

normalize_sbom() {
  input_file="$1"
  output_file="$2"

  python3 - "$input_file" "$output_file" "$ROOT_DIR" <<'PY'
import json
import os
import sys

input_file, output_file, root_dir = sys.argv[1:]
root_dir = root_dir.rstrip("/")
root_prefixes = {root_dir, os.path.realpath(root_dir)}
root_file_uris = ["file://" + path.rstrip("/") for path in sorted(root_prefixes, key=len, reverse=True)]


def normalize(value):
    if isinstance(value, str):
        for root_file_uri in root_file_uris:
            value = value.replace("path+" + root_file_uri, "path+repo://libaprs-engine")
            value = value.replace(root_file_uri, "file://.")
        return value
    if isinstance(value, list):
        return [normalize(item) for item in value]
    if isinstance(value, dict):
        return {key: normalize(item) for key, item in value.items()}
    return value


with open(input_file, "r", encoding="utf-8") as handle:
    data = json.load(handle)

with open(output_file, "w", encoding="utf-8") as handle:
    json.dump(normalize(data), handle, indent=2)
    handle.write("\n")
PY
}

write_workspace_crates() {
  output_file="$1"

  cargo metadata --no-deps --format-version 1 \
    | python3 -c '
import json
import os
import sys

root_dir = os.path.realpath(sys.argv[1])
metadata = json.load(sys.stdin)
members = set(metadata["workspace_members"])

for package in sorted(metadata["packages"], key=lambda item: item["manifest_path"]):
    if package["id"] not in members:
        continue
    name = package["name"]
    manifest = os.path.realpath(package["manifest_path"])
    manifest_dir = os.path.dirname(manifest)
    rel_manifest = os.path.relpath(manifest, root_dir)
    print(f"{name}\t{manifest_dir}\t{rel_manifest}")
' "$ROOT_DIR" >"$output_file"
}

cleanup_workspace_sbom_temps() {
  tmp_prefix="$1"
  workspace_crates_file="$2"

  while IFS="$(printf '\t')" read -r _ manifest_dir _; do
    [ -n "$manifest_dir" ] || continue
    rm -f "$manifest_dir/$tmp_prefix.json"
  done <"$workspace_crates_file"
}

workspace_manifest_paths() {
  tmp_file="$1"

  cut -f3 "$tmp_file"
}

hash_inputs() {
  evidence_base="$1"

  {
    printf '%s\n' \
      Cargo.lock \
      Cargo.toml \
      .github/dependabot.yml \
      deny.toml \
      docs/release.md \
      docs/supply-chain.md

    (
      cd "$ROOT_DIR"
      find .github/workflows -maxdepth 1 -type f \( -name '*.yml' -o -name '*.yaml' \) -print
      find examples/downstream-smoke -maxdepth 1 -type f \( -name Cargo.toml -o -name Cargo.lock \) -print
      find fuzz -maxdepth 1 -type f \( -name Cargo.toml -o -name Cargo.lock \) -print
      find scripts -maxdepth 1 -type f -name '*.sh' -print
    )

    if [ -d "$evidence_base/supply-chain" ]; then
      (
        cd "$evidence_base"
        find supply-chain -maxdepth 1 -type f ! -name SHA256SUMS -print 2>/dev/null || true
        find supply-chain/sbom -maxdepth 1 -type f -name '*.cdx.json' -print 2>/dev/null || true
      )
    fi
  } | sed 's#^\./##' | LC_ALL=C sort -u
}

generate_sboms() {
  output_root="$1"
  tmp_root="$2"
  tmp_prefix=".workspace.supply-chain.$$"
  workspace_crates_file="$tmp_root/workspace-crates.tsv"

  write_workspace_crates "$workspace_crates_file"
  cleanup_workspace_sbom_temps "$tmp_prefix" "$workspace_crates_file"
  rm -rf "$output_root/sbom"
  mkdir -p "$output_root/sbom"

  (
    trap 'cleanup_workspace_sbom_temps "$tmp_prefix" "$workspace_crates_file"' EXIT HUP INT TERM
    cd "$ROOT_DIR"
    SOURCE_DATE_EPOCH=0 cargo cyclonedx \
      --manifest-path Cargo.toml \
      --format json \
      --spec-version 1.5 \
      --target all \
      --all-features \
      --license-strict \
      --override-filename "$tmp_prefix" >/dev/null

    while IFS="$(printf '\t')" read -r name manifest_dir _; do
        [ -n "$name" ] || continue
        tmp_json="$manifest_dir/$tmp_prefix.json"

        [ -f "$tmp_json" ] || fail "cargo-cyclonedx did not create SBOM for $name"
        normalize_sbom "$tmp_json" "$tmp_root/$name.cdx.json"
        rm -f "$tmp_json"
        mv "$tmp_root/$name.cdx.json" "$output_root/sbom/$name.cdx.json"
      done <"$workspace_crates_file"
  )
}

generate_evidence() {
  evidence_base="$1"
  output_root="$evidence_base/supply-chain"
  tmp_root="$2"

  mkdir -p "$output_root" "$tmp_root"
  workspace_crates_file="$tmp_root/hash-workspace-crates.tsv"
  write_workspace_crates "$workspace_crates_file"
  if [ "$evidence_base" != "$ROOT_DIR" ] && [ -d "$ROOT_DIR/supply-chain" ]; then
    find "$ROOT_DIR/supply-chain" -maxdepth 1 -type f ! -name SHA256SUMS -print \
      | while IFS= read -r static_file; do
          cp "$static_file" "$output_root/$(basename -- "$static_file")"
        done
  fi
  generate_sboms "$output_root" "$tmp_root"
  : >"$output_root/SHA256SUMS"
  {
    workspace_manifest_paths "$workspace_crates_file"
    hash_inputs "$evidence_base"
  } | LC_ALL=C sort -u | while IFS= read -r path; do
    [ -n "$path" ] || continue

    case "$path" in
      supply-chain/*)
        file="$evidence_base/$path"
        ;;
      *)
        file="$ROOT_DIR/$path"
        ;;
    esac

    [ -f "$file" ] || fail "hash input does not exist: $path"
    printf '%s  %s\n' "$(sha256_file "$file")" "$path" >>"$output_root/SHA256SUMS"
  done

  if grep -R "$ROOT_DIR" "$output_root" >/dev/null 2>&1; then
    fail "generated evidence contains local absolute path: $ROOT_DIR"
  fi
}

check_mode=0
case "${1:-}" in
  "")
    ;;
  --check)
    check_mode=1
    ;;
  -h|--help)
    usage
    exit 0
    ;;
  *)
    usage
    exit 2
    ;;
esac

has_tool cargo-cyclonedx || fail "cargo-cyclonedx is required; run: cargo install cargo-cyclonedx --version 0.5.9 --locked"
has_tool python3 || fail "python3 is required to normalize SBOM JSON"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT HUP INT TERM

if [ "$check_mode" -eq 1 ]; then
  generate_evidence "$tmp_dir" "$tmp_dir/normalize"
  if ! diff -ru "$ROOT_DIR/supply-chain" "$tmp_dir/supply-chain"; then
    fail "tracked supply-chain evidence is stale; run scripts/generate-supply-chain-evidence.sh"
  fi
else
  generate_evidence "$ROOT_DIR" "$tmp_dir/normalize"
fi

echo "supply-chain evidence generated"
