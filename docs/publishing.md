# Publishing

![libaprs-engine documentation header](assets/brand/docs-header.svg)

This repository is ready for crates.io package validation, but publishing
requires a crates.io account token and must be done only from a clean, verified
release commit after secure review has passed with no findings.

## Crates

Publish crates in dependency order:

1. `libaprs-engine`
2. `aprs-transport-file`
3. `aprs-transport-tcp`
4. `aprs-transport-aprs-is`
5. `aprs-transport-async`
6. `aprs-transport-ax25`
7. `aprs-transport-channel`
8. `aprs-transport-corpus`
9. `aprs-transport-file-watch`
10. `aprs-transport-http`
11. `aprs-transport-kiss`
12. `aprs-transport-mqtt`
13. `aprs-transport-serial`
14. `aprs-transport-udp`
15. `aprs-cli`

The adapter and CLI crates use versioned path dependencies:

```toml
libaprs-engine = { version = "3.0.0-rc.2", path = "../libaprs-engine" }
```

Cargo uses the local path in this workspace and the version requirement when
packaging for crates.io.

## Dry Run

Run package validation before publishing the core crate:

```sh
cargo package -p libaprs-engine
```

Before `libaprs-engine` exists on crates.io, Cargo cannot package dependent
crates because their registry dependency is not resolvable yet. After
`libaprs-engine` is published and visible in the crates.io index, validate the
dependent crates:

```sh
cargo package -p aprs-transport-file
cargo package -p aprs-transport-tcp
cargo package -p aprs-transport-aprs-is
cargo package -p aprs-transport-async
cargo package -p aprs-transport-ax25
cargo package -p aprs-transport-channel
cargo package -p aprs-transport-corpus
cargo package -p aprs-transport-file-watch
cargo package -p aprs-transport-http
cargo package -p aprs-transport-kiss
cargo package -p aprs-transport-mqtt
cargo package -p aprs-transport-serial
cargo package -p aprs-transport-udp
cargo package -p aprs-cli
```

## Publish

Authenticate with crates.io outside the repository:

```sh
cargo login
```

The repository provides a guarded publish script that refuses to run unless
publishing is explicitly confirmed, the working tree is clean, the release
commit is identified, and pre-publish evidence confirms clean secure review and
passing release/security gates. The same script also creates or updates the
GitHub Release. Stable releases are marked latest and verified as latest:

```sh
LIBAPRS_CONFIRM_PUBLISH=1 \
LIBAPRS_SECURE_REVIEW=clean \
LIBAPRS_LOCAL_RELEASE_GATE=passed \
LIBAPRS_SECURITY_GATE=passed \
LIBAPRS_REMOTE_CI=passed \
LIBAPRS_GITHUB_RELEASE=publish \
LIBAPRS_RELEASE_TAG=v1.0.1 \
LIBAPRS_GITHUB_REPO=SoloSentryOrg/libaprs-engine \
LIBAPRS_RELEASE_COMMIT="$(git rev-parse HEAD)" \
  scripts/publish-release.sh
```

Use `LIBAPRS_REMOTE_CI=skipped-documented` only when GitHub Actions is blocked
or intentionally skipped and the release notes document the reason. Do not use
that override to bypass a failing CI run.

Use `LIBAPRS_GITHUB_RELEASE=skipped-documented` only when GitHub Releases are
unavailable and the release evidence records the reason. Do not use that
override for a normal release. Set `LIBAPRS_GITHUB_RELEASE_NOTES_FILE=<path>`
to publish curated release notes; otherwise the script asks GitHub to generate
notes for the tag.

For release-candidate or other prerelease tags, set
`LIBAPRS_GITHUB_RELEASE_PRERELEASE=1`. The script marks the GitHub Release as a
prerelease, passes `--latest=false` when creating it, and verifies the tag did
not replace the stable latest release:

```sh
LIBAPRS_CONFIRM_PUBLISH=1 \
LIBAPRS_SECURE_REVIEW=clean \
LIBAPRS_LOCAL_RELEASE_GATE=passed \
LIBAPRS_SECURITY_GATE=passed \
LIBAPRS_REMOTE_CI=passed \
LIBAPRS_GITHUB_RELEASE=publish \
LIBAPRS_GITHUB_RELEASE_PRERELEASE=1 \
LIBAPRS_RELEASE_TAG=v3.0.0-rc.2 \
LIBAPRS_GITHUB_REPO=SoloSentryOrg/libaprs-engine \
LIBAPRS_RELEASE_COMMIT="$(git rev-parse HEAD)" \
LIBAPRS_GITHUB_RELEASE_NOTES_FILE=docs/release-notes-v3.0.0-rc.2.md \
  scripts/publish-release.sh
```

Use the default Cargo home for normal publication. In restricted environments
where `~/.cargo` is not writable, keep Cargo state outside the repository:

```sh
mkdir -p /tmp/libaprs-cargo-home
CARGO_HOME=/tmp/libaprs-cargo-home \
  LIBAPRS_COPY_CARGO_CREDENTIALS=1 \
  LIBAPRS_CONFIRM_PUBLISH=1 \
  LIBAPRS_SECURE_REVIEW=clean \
  LIBAPRS_LOCAL_RELEASE_GATE=passed \
  LIBAPRS_SECURITY_GATE=passed \
  LIBAPRS_REMOTE_CI=passed \
  LIBAPRS_GITHUB_RELEASE=publish \
  LIBAPRS_RELEASE_TAG=v1.0.1 \
  LIBAPRS_GITHUB_REPO=SoloSentryOrg/libaprs-engine \
  LIBAPRS_RELEASE_COMMIT="$(git rev-parse HEAD)" \
  scripts/publish-release.sh
```

Do not commit Cargo credentials, registry caches, package caches, advisory
databases, or temporary Cargo homes.

If publishing manually, publish in dependency order:

```sh
cargo publish -p libaprs-engine
cargo publish -p aprs-transport-file
cargo publish -p aprs-transport-tcp
cargo publish -p aprs-transport-aprs-is
cargo publish -p aprs-transport-async
cargo publish -p aprs-transport-ax25
cargo publish -p aprs-transport-channel
cargo publish -p aprs-transport-corpus
cargo publish -p aprs-transport-file-watch
cargo publish -p aprs-transport-http
cargo publish -p aprs-transport-kiss
cargo publish -p aprs-transport-mqtt
cargo publish -p aprs-transport-serial
cargo publish -p aprs-transport-udp
cargo publish -p aprs-cli
```

Wait for each published dependency to become available before publishing crates
that depend on it.

Manual publishing must still satisfy the same pre-publish evidence requirements
as `scripts/publish-release.sh`: clean secure review, passing local release
gate, passing security gate, passing remote CI or documented CI skip, clean
working tree, identified release commit, pushed release tag, and GitHub Release
creation or documented GitHub Release skip.

## Release Requirements

- Run the local verification gate in `docs/verification.md`.
- Confirm GitHub Actions passed on the release commit.
- Confirm secure code review passed cleanly with no findings.
- Confirm `cargo audit` and `cargo deny check` passed locally or in the
  security workflow.
- Run `cargo package -p libaprs-engine`.
- After `libaprs-engine` is published, run package validation for dependent
  crates before publishing them.
- Confirm `CHANGELOG.md` describes the release.
- Tag only after package validation and CI both pass.
- Confirm the GitHub Release for a stable release tag is marked latest before
  closing the release. Confirm release-candidate tags are marked prerelease and
  do not replace the stable latest release.
- Update GitHub Project #3 and the `ROADMAP.md` backup snapshot after every
  release before treating the release as complete.
