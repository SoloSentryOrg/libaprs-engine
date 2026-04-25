# Release Checklist

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
- Confirm GitHub Actions is enabled before relying on remote CI.
- Tag the release after local and remote verification pass.
