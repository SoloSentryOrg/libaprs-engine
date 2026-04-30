#!/usr/bin/env sh
set -eu

manifest="${1:-examples/downstream-smoke/Cargo.toml}"
version_manifest="${2:-crates/libaprs-engine/Cargo.toml}"

version="$(
  awk -F '"' '
    $1 ~ /^version = / {
      print $2
      exit
    }
  ' "$version_manifest"
)"

if [ -z "$version" ]; then
  echo "$version_manifest: unable to determine workspace release version" >&2
  exit 1
fi

awk -v expected="$version" '
  /^[[:space:]]*libaprs-engine = / || /^[[:space:]]*aprs-transport-/ {
    declared = $0
    if (declared ~ /version = "/) {
      sub(/^.*version = "/, "", declared)
    } else {
      sub(/^.*= "/, "", declared)
    }
    sub(/".*$/, "", declared)

    if (declared != expected) {
      printf "%s:%d: downstream smoke dependency must use workspace release version %s: %s\n", FILENAME, FNR, expected, $0 > "/dev/stderr"
      failed = 1
    }
  }
  END { exit failed ? 1 : 0 }
' "$manifest"
