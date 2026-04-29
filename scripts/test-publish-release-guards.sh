#!/usr/bin/env sh
set -eu

ROOT_DIR="$(
  CDPATH=
  cd -- "$(dirname -- "$0")/.." && pwd
)"

fail() {
  echo "test-publish-release-guards: $*" >&2
  exit 1
}

make_stubs() {
  stub_dir="$1"
  mkdir -p "$stub_dir"

  cat >"$stub_dir/git" <<'STUB'
#!/usr/bin/env sh
set -eu
case "$1" in
  status)
    exit 0
    ;;
  rev-parse)
    if [ "${2:-}" = "HEAD" ]; then
      printf '%s\n' "${LIBAPRS_TEST_HEAD:-test-head}"
      exit 0
    fi
    ;;
esac
echo "unexpected git invocation: $*" >&2
exit 99
STUB

  cat >"$stub_dir/cargo" <<'STUB'
#!/usr/bin/env sh
set -eu
if [ "$1" = "publish" ] && [ "$2" = "-p" ]; then
  printf '%s\n' "$3" >>"${LIBAPRS_TEST_PUBLISHED:?}"
  exit 0
fi
echo "unexpected cargo invocation: $*" >&2
exit 99
STUB

  cat >"$stub_dir/gh" <<'STUB'
#!/usr/bin/env sh
set -eu
if [ "$1" = "release" ] && [ "$2" = "view" ]; then
  exit 1
fi
if [ "$1" = "release" ] && [ "$2" = "create" ]; then
  printf '%s\n' "$3" >>"${LIBAPRS_TEST_RELEASED:?}"
  exit 0
fi
if [ "$1" = "release" ] && [ "$2" = "list" ]; then
  printf '%s\n' "${LIBAPRS_TEST_RELEASE_TAG:?}"
  exit 0
fi
echo "unexpected gh invocation: $*" >&2
exit 99
STUB

  chmod +x "$stub_dir/git" "$stub_dir/cargo" "$stub_dir/gh"
}

run_publish() {
  env -i \
    PATH="$STUB_DIR:/bin:/usr/bin" \
    LIBAPRS_TEST_HEAD="$LIBAPRS_TEST_HEAD" \
    LIBAPRS_TEST_PUBLISHED="$LIBAPRS_TEST_PUBLISHED" \
    LIBAPRS_TEST_RELEASED="$LIBAPRS_TEST_RELEASED" \
    LIBAPRS_TEST_RELEASE_TAG="$LIBAPRS_TEST_RELEASE_TAG" \
    "$@" \
    "$ROOT_DIR/scripts/publish-release.sh"
}

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT
STUB_DIR="$TMP_DIR/bin"
LIBAPRS_TEST_PUBLISHED="$TMP_DIR/published"
LIBAPRS_TEST_RELEASED="$TMP_DIR/released"
LIBAPRS_TEST_HEAD="abc123"
LIBAPRS_TEST_RELEASE_TAG="v9.9.9"
export STUB_DIR LIBAPRS_TEST_PUBLISHED LIBAPRS_TEST_RELEASED LIBAPRS_TEST_HEAD LIBAPRS_TEST_RELEASE_TAG
make_stubs "$STUB_DIR"

if run_publish LIBAPRS_CONFIRM_PUBLISH=1 >"$TMP_DIR/fail.out" 2>&1; then
  fail "publish succeeded without secure review evidence"
fi

if [ -s "$LIBAPRS_TEST_PUBLISHED" ]; then
  fail "publish attempted before secure review evidence was accepted"
fi

if ! grep -qi "secure review" "$TMP_DIR/fail.out"; then
  fail "missing secure review failure message"
fi

if run_publish \
  LIBAPRS_CONFIRM_PUBLISH=1 \
  LIBAPRS_SECURE_REVIEW=clean \
  LIBAPRS_LOCAL_RELEASE_GATE=passed \
  LIBAPRS_SECURITY_GATE=passed \
  LIBAPRS_REMOTE_CI=passed \
  LIBAPRS_RELEASE_COMMIT=abc123 \
  >"$TMP_DIR/missing-release.out" 2>&1; then
  fail "publish succeeded without GitHub release evidence"
fi

if ! grep -qi "GitHub release" "$TMP_DIR/missing-release.out"; then
  fail "missing GitHub release failure message"
fi

run_publish \
  LIBAPRS_CONFIRM_PUBLISH=1 \
  LIBAPRS_SECURE_REVIEW=clean \
  LIBAPRS_LOCAL_RELEASE_GATE=passed \
  LIBAPRS_SECURITY_GATE=passed \
  LIBAPRS_REMOTE_CI=passed \
  LIBAPRS_RELEASE_COMMIT=abc123 \
  LIBAPRS_GITHUB_RELEASE=publish \
  LIBAPRS_RELEASE_TAG=v9.9.9 \
  LIBAPRS_GITHUB_REPO=example/libaprs-engine \
  >"$TMP_DIR/pass.out" 2>&1

published_count="$(wc -l <"$LIBAPRS_TEST_PUBLISHED" | tr -d ' ')"
[ "$published_count" = "15" ] || fail "expected 15 publish calls, got $published_count"

released_count="$(wc -l <"$LIBAPRS_TEST_RELEASED" | tr -d ' ')"
[ "$released_count" = "1" ] || fail "expected 1 GitHub release call, got $released_count"

echo "publish release guard tests passed"
