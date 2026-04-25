# Release Checklist

## Local Gate

- Run `cargo fmt --all --check`.
- Run `cargo test`.
- Run `cargo test --all-features`.
- Run `cargo test --examples`.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.
- Run `cargo metadata --no-deps --format-version 1`.
- Run `cargo doc --no-deps --all-features`.
- Run `cargo bench -p libaprs-engine` when parser performance changed.
- Verify the declared MSRV with `cargo +1.80.0 test --all-features`.
- Consider `cargo audit` or `cargo deny check` when dependency changes are part
  of the release.
- Confirm `CHANGELOG.md` describes the release.

## Remote Gate

- Confirm GitHub Actions is enabled before relying on remote CI.
- If GitHub Actions is blocked before job creation, document that CI was skipped
  and rely on the local gate.

## Tagging

- Tag the release after local and remote verification pass.
- If remote CI is intentionally skipped, tag only after the local gate passes
  and the skipped remote gate is documented.

## v0.1.0 Release Evidence

- Tag: `v0.1.0`
- Commit: `043630419453548cfdc026410788d89ba00386b7`
- Local gate passed.
- Remote GitHub Actions was skipped because startup is blocked before job
  creation by account, billing, or policy state outside this repository.
