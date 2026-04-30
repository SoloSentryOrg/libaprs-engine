#!/usr/bin/env sh
set -eu

ROOT="$(CDPATH=; cd -- "$(dirname -- "$0")/.." && pwd)"
SCRIPT="$ROOT/scripts/check-fuzz-corpus.sh"

TMPDIR="${TMPDIR:-/tmp}"
WORKDIR="$(mktemp -d "$TMPDIR/libaprs-fuzz-corpus-test.XXXXXX")"
trap 'rm -rf "$WORKDIR"' EXIT INT HUP TERM

mkdir -p "$WORKDIR/fuzz/corpus/parse_packet"
printf 'N0CALL>APRS:>ok' >"$WORKDIR/fuzz/corpus/parse_packet/status"

"$SCRIPT" "$WORKDIR/fuzz/corpus" >/dev/null

dd if=/dev/zero of="$WORKDIR/fuzz/corpus/parse_packet/oversized" bs=1 count=4097 >/dev/null 2>&1
if "$SCRIPT" "$WORKDIR/fuzz/corpus" >/dev/null 2>&1; then
  echo "expected oversized corpus file to fail" >&2
  exit 1
fi
rm "$WORKDIR/fuzz/corpus/parse_packet/oversized"

printf 'private-token' >"$WORKDIR/fuzz/corpus/parse_packet/.secret"
if "$SCRIPT" "$WORKDIR/fuzz/corpus" >/dev/null 2>&1; then
  echo "expected hidden corpus file to fail" >&2
  exit 1
fi
rm "$WORKDIR/fuzz/corpus/parse_packet/.secret"

mkdir "$WORKDIR/fuzz/corpus/parse_packet/.artifacts"
printf 'private-token' >"$WORKDIR/fuzz/corpus/parse_packet/.artifacts/token"
if "$SCRIPT" "$WORKDIR/fuzz/corpus" >/dev/null 2>&1; then
  echo "expected hidden corpus directory to fail" >&2
  exit 1
fi

echo "fuzz corpus guard tests passed"
