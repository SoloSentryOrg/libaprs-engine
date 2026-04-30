#!/usr/bin/env sh
set -eu

CORPUS_DIR="${1:-fuzz/corpus}"
MAX_BYTES="${LIBAPRS_MAX_FUZZ_CORPUS_BYTES:-4096}"

if [ ! -d "$CORPUS_DIR" ]; then
  echo "fuzz corpus directory not found: $CORPUS_DIR" >&2
  exit 1
fi

unsafe_path="$(
  find "$CORPUS_DIR" \
    \( -name '.*' \
    -o -name '*.tmp' \
    -o -name '*.log' \
    -o -name '*.profraw' \
    -o -name 'crash-*' \
    -o -name 'timeout-*' \
    -o -name 'oom-*' \
    -o -name 'leak-*' \
    -o -name 'artifact*' \) \
    -print -quit
)"

if [ -n "$unsafe_path" ]; then
  echo "unsafe fuzz corpus artifact path: $unsafe_path" >&2
  exit 1
fi

find "$CORPUS_DIR" -type f -exec sh -c '
max_bytes="$1"
shift
for file do
  size="$(wc -c <"$file" | tr -d " ")"
  if [ "$size" -gt "$max_bytes" ]; then
    echo "fuzz corpus file exceeds ${max_bytes} bytes: $file ($size bytes)" >&2
    exit 1
  fi
done
' sh "$MAX_BYTES" {} +

echo "fuzz corpus guard passed"
