# Release Checklist

## Local Gate

- Run `cargo fmt --all --check`.
- Run `cargo test`.
- Run `cargo test --all-features`.
- Run `cargo test --examples`.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.
- Run `cargo metadata --no-deps --format-version 1`.
- Run `cargo doc --no-deps --all-features`.
- Run `cargo package -p libaprs-engine`.
- After `libaprs-engine` is published to crates.io, run package validation for
  dependent crates before publishing them.
- Run `cargo bench -p libaprs-engine` when parser performance changed.
- Verify the declared MSRV with `cargo +1.80.0 test --all-features`.
- Consider `cargo audit` or `cargo deny check` when dependency changes are part
  of the release.
- Confirm `CHANGELOG.md` describes the release.
- Confirm crate publish order in `docs/publishing.md` when publishing to
  crates.io.

## Remote Gate

- Confirm GitHub Actions is enabled before relying on remote CI.
- If GitHub Actions is blocked before job creation, document that CI was skipped
  and rely on the local gate.

## Tagging

- Tag the release after local and remote verification pass.
- If remote CI is intentionally skipped, tag only after the local gate passes
  and the skipped remote gate is documented.

## v0.1.2 Release Evidence

- Tag: `v0.1.2`
- Commit: release tag target (`git rev-list -n 1 v0.1.2`).
- Local gate: passed on 2026-04-25.
- Remote GitHub Actions: passed on 2026-04-25 before tag creation.
- Notes: expands APRS semantics with telemetry metadata, NMEA checksum
  inspection, Mic-E coordinate/speed/course helpers, and explicit nested
  third-party parsing.

## v0.1.1 Release Evidence

- Tag: `v0.1.1`
- Commit: release tag target (`git rev-list -n 1 v0.1.1`).
- Local gate: passed on 2026-04-25.
- Remote GitHub Actions: passed on 2026-04-25:
  <https://github.com/elodiejmirza/libaprs-engine/actions/runs/24937390267>.
- Notes: supersedes `v0.1.0` with MSRV clippy compatibility and Node.js
  24-compatible checkout action. Superseded by `v0.1.2`.

## v0.1.0 Release Evidence

- Tag: `v0.1.0`
- Commit: `043630419453548cfdc026410788d89ba00386b7`
- Local gate passed.
- Remote GitHub Actions was skipped because startup is blocked before job
  creation by account, billing, or policy state outside this repository.
- Superseded by `v0.1.1` and `v0.1.2`.
