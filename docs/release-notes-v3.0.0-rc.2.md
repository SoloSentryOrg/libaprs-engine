# libaprs-engine v3.0.0-rc.2

## BLUF

- `v3.0.0-rc.2` is a security fix-forward release candidate for `v3.0.0`.
- It supersedes `v3.0.0-rc.1`, which was published before the final
  repo-wide security hardening pass.
- No intentional public API break is approved in this release candidate.
- Existing `2.6.0` and `3.0.0-rc.1` users should not need source changes for
  the RC, but TCP helper defaults now use finite timeouts.
- The RC must pass local release gates, remote CI, security checks, semver
  review, downstream smoke, and supply-chain evidence checks before final
  `v3.0.0`.

## Changes Since `v3.0.0-rc.1`

- Escaped user-controlled CLI diagnostic values for unknown options, invalid
  `--fail-on` values, and input path read/open errors.
- Hardened APRS-IS login/profile helpers to reject all ASCII control bytes, not
  only CR and LF.
- Added finite default TCP connect/read timeouts while preserving explicit
  caller override through `TcpReadOptions`.
- Updated APRS-IS examples and transport docs to prefer `profile_line()` for
  untrusted login fields.
- Added tracked security-audit outcome evidence for the fix-forward pass.
- Bumped all workspace crates to `3.0.0-rc.2`.

## Security Notes

- Packet and transport input remains untrusted.
- Accepted packets must preserve raw bytes exactly.
- Malformed packet shape must fail closed.
- Callers that intentionally require indefinite TCP blocking can set
  `TcpReadOptions` timeout fields to `None`.
- Post-publication downstream smoke must regenerate its lockfile from crates.io
  so checksum evidence matches the published RC crates.
- The RC fix-forward audit summary is tracked in
  [v3.0.0-rc.2 Security Audit Summary](security-audit-v3.0.0-rc.2.md).

## Release Gates

- `scripts/verify-docs.sh`
- `scripts/test-supply-chain-evidence.sh`
- `scripts/verify-release.sh`
- `cargo semver-checks check-release -p libaprs-engine` when available
- Post-publication downstream smoke against `3.0.0-rc.2`

## Upgrade Notes

Existing `2.6.0` users can test the release candidate by updating dependency
versions to `=3.0.0-rc.2`. No source changes are expected. TCP address-helper
callers that relied on indefinite blocking should explicitly set
`connect_timeout` or `read_timeout` to `None`.
