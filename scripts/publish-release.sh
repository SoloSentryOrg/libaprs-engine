#!/usr/bin/env sh
set -eu

run() {
  printf '+ %s\n' "$*" >&2
  "$@"
}

require_value() {
  name="$1"
  actual="$2"
  expected="$3"
  description="$4"

  if [ "$actual" != "$expected" ]; then
    echo "Refusing to publish: $description requires $name=$expected." >&2
    exit 1
  fi
}

if [ "${LIBAPRS_CONFIRM_PUBLISH:-0}" != "1" ]; then
  echo "Set LIBAPRS_CONFIRM_PUBLISH=1 to publish crates to crates.io." >&2
  exit 2
fi

require_value LIBAPRS_SECURE_REVIEW "${LIBAPRS_SECURE_REVIEW:-}" clean "secure review"
require_value LIBAPRS_LOCAL_RELEASE_GATE "${LIBAPRS_LOCAL_RELEASE_GATE:-}" passed "local release gate"
require_value LIBAPRS_SECURITY_GATE "${LIBAPRS_SECURITY_GATE:-}" passed "security gate"

case "${LIBAPRS_REMOTE_CI:-}" in
  passed | skipped-documented)
    ;;
  *)
    echo "Refusing to publish: remote CI requires LIBAPRS_REMOTE_CI=passed or LIBAPRS_REMOTE_CI=skipped-documented." >&2
    exit 1
    ;;
esac

if [ -z "${LIBAPRS_RELEASE_COMMIT:-}" ]; then
  echo "Refusing to publish: release commit requires LIBAPRS_RELEASE_COMMIT=<git commit>." >&2
  exit 1
fi

if [ "$(git rev-parse HEAD)" != "$LIBAPRS_RELEASE_COMMIT" ]; then
  echo "Refusing to publish: LIBAPRS_RELEASE_COMMIT does not match HEAD." >&2
  exit 1
fi

if [ -n "$(git status --short)" ]; then
  echo "Refusing to publish from a dirty working tree." >&2
  exit 1
fi

if [ "${LIBAPRS_COPY_CARGO_CREDENTIALS:-0}" = "1" ]; then
  : "${CARGO_HOME:?CARGO_HOME must be set when copying credentials}"
  mkdir -p "$CARGO_HOME"
  install -m 600 "$HOME/.cargo/credentials.toml" "$CARGO_HOME/credentials.toml"
fi

publish_crate() {
  run cargo publish -p "$1"
}

publish_crate libaprs-engine
publish_crate aprs-transport-file
publish_crate aprs-transport-tcp
publish_crate aprs-transport-aprs-is
publish_crate aprs-transport-async
publish_crate aprs-transport-ax25
publish_crate aprs-transport-channel
publish_crate aprs-transport-corpus
publish_crate aprs-transport-file-watch
publish_crate aprs-transport-http
publish_crate aprs-transport-kiss
publish_crate aprs-transport-mqtt
publish_crate aprs-transport-serial
publish_crate aprs-transport-udp
publish_crate aprs-cli
