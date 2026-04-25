# Release Checklist

- Run `cargo test`.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.
- Run `cargo metadata --no-deps --format-version 1`.
- Confirm `CHANGELOG.md` describes the release.
- Confirm GitHub Actions is enabled before relying on remote CI.
- Tag the release after local and remote verification pass.
