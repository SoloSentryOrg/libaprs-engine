# libaprs-engine v3.0.0-rc.1

## BLUF

- `v3.0.0-rc.1` is the first release candidate for the next major release line.
- No intentional public API break is approved in this release candidate.
- Existing `2.6.0` users should only need dependency-version changes for RC
  testing.
- Raw-byte preservation, fail-closed malformed packet handling, bounded
  transport input, and caller-owned runtime policy remain unchanged.
- The RC must pass local release gates, remote CI, security checks, semver
  review, downstream smoke, and supply-chain evidence checks before final
  `v3.0.0`.

## Changes

- Bumped all workspace crates to `3.0.0-rc.1`.
- Updated current README, API, transport, publishing, APRS-IS profile, and
  downstream-smoke examples to target `3.0.0-rc.1`.
- Added `v3.0.0` migration guidance that documents the no-intentional-break
  migration from `2.6.0`.
- Updated the `v3.0.0` breaking-change decision record so future breaking
  changes remain evidence-gated.
- Added a docs gate requiring RC release notes, migration guidance, and release
  preparation evidence to stay in sync.

## Security Notes

- Packet and transport input remains untrusted.
- Accepted packets must preserve raw bytes exactly.
- Malformed packet shape must fail closed.
- Applications should pin the exact RC version during testing and report any
  source, semver, or behavior regression before final `3.0.0`.
- Post-publication downstream smoke must regenerate its lockfile from crates.io
  so checksum evidence matches the published RC crates.
- The RC fix-forward audit summary is tracked in
  [v3.0.0-rc.1 Security Audit Summary](security-audit-v3.0.0-rc.1.md).

## Release Gates

- `scripts/verify-docs.sh`
- `scripts/test-supply-chain-evidence.sh`
- `scripts/verify-release.sh`
- `cargo semver-checks check-release -p libaprs-engine` when available
- Post-publication downstream smoke against `3.0.0-rc.1`

## Upgrade Notes

Existing `2.6.0` users can test the release candidate by updating dependency
versions to `=3.0.0-rc.1`. No source changes are expected.
