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

is_truthy() {
  case "$1" in
    1 | true | TRUE | yes | YES)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
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

case "${LIBAPRS_GITHUB_RELEASE:-}" in
  publish)
    if [ -z "${LIBAPRS_RELEASE_TAG:-}" ]; then
      echo "Refusing to publish: GitHub release publication requires LIBAPRS_RELEASE_TAG=<tag>." >&2
      exit 1
    fi
    if [ -z "${LIBAPRS_GITHUB_REPO:-}" ]; then
      echo "Refusing to publish: GitHub release publication requires LIBAPRS_GITHUB_REPO=<owner/name>." >&2
      exit 1
    fi
    case "$LIBAPRS_RELEASE_TAG" in
      *-*)
        if ! is_truthy "${LIBAPRS_GITHUB_RELEASE_PRERELEASE:-0}"; then
          echo "Refusing to publish: prerelease tags require LIBAPRS_GITHUB_RELEASE_PRERELEASE=1." >&2
          exit 1
        fi
        ;;
    esac
    ;;
  skipped-documented)
    ;;
  *)
    echo "Refusing to publish: GitHub release requires LIBAPRS_GITHUB_RELEASE=publish or LIBAPRS_GITHUB_RELEASE=skipped-documented." >&2
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

publish_github_release() {
  tag="$LIBAPRS_RELEASE_TAG"
  repo="$LIBAPRS_GITHUB_REPO"
  title="${LIBAPRS_GITHUB_RELEASE_TITLE:-libaprs-engine $tag}"
  prerelease=0

  if is_truthy "${LIBAPRS_GITHUB_RELEASE_PRERELEASE:-0}"; then
    prerelease=1
  fi

  if gh release view "$tag" --repo "$repo" >/dev/null 2>&1; then
    if [ "$prerelease" = "1" ]; then
      if [ -n "${LIBAPRS_GITHUB_RELEASE_NOTES_FILE:-}" ]; then
        run gh release edit "$tag" --repo "$repo" --title "$title" --prerelease --verify-tag --notes-file "$LIBAPRS_GITHUB_RELEASE_NOTES_FILE"
      else
        run gh release edit "$tag" --repo "$repo" --title "$title" --prerelease --verify-tag
      fi
    else
      if [ -n "${LIBAPRS_GITHUB_RELEASE_NOTES_FILE:-}" ]; then
        run gh release edit "$tag" --repo "$repo" --title "$title" --latest --verify-tag --notes-file "$LIBAPRS_GITHUB_RELEASE_NOTES_FILE"
      else
        run gh release edit "$tag" --repo "$repo" --title "$title" --latest --verify-tag
      fi
    fi
  elif [ -n "${LIBAPRS_GITHUB_RELEASE_NOTES_FILE:-}" ]; then
    if [ "$prerelease" = "1" ]; then
      run gh release create "$tag" --repo "$repo" --title "$title" --prerelease --latest=false --verify-tag --notes-file "$LIBAPRS_GITHUB_RELEASE_NOTES_FILE"
    else
      run gh release create "$tag" --repo "$repo" --title "$title" --latest --verify-tag --notes-file "$LIBAPRS_GITHUB_RELEASE_NOTES_FILE"
    fi
  else
    if [ "$prerelease" = "1" ]; then
      run gh release create "$tag" --repo "$repo" --title "$title" --prerelease --latest=false --verify-tag --generate-notes
    else
      run gh release create "$tag" --repo "$repo" --title "$title" --latest --verify-tag --generate-notes
    fi
  fi

  latest_tag="$(gh release list --repo "$repo" --limit 100 --json tagName,isLatest --jq '.[] | select(.isLatest == true) | .tagName')"
  if [ "$prerelease" = "1" ]; then
    release_is_prerelease="$(gh release view "$tag" --repo "$repo" --json isPrerelease --jq '.isPrerelease')"
    if [ "$release_is_prerelease" != "true" ]; then
      echo "Refusing to finish: GitHub release '$tag' is not marked prerelease." >&2
      exit 1
    fi
    if [ "$latest_tag" = "$tag" ]; then
      echo "Refusing to finish: GitHub prerelease '$tag' must not be marked latest." >&2
      exit 1
    fi
  else
    if [ "$latest_tag" != "$tag" ]; then
      echo "Refusing to finish: GitHub latest release is '$latest_tag', expected '$tag'." >&2
      exit 1
    fi
  fi
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

if [ "${LIBAPRS_GITHUB_RELEASE:-}" = "publish" ]; then
  publish_github_release
fi
