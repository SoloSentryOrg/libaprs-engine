# v3.0.0-rc.1 Security Audit Summary

## BLUF

- Repo-wide source security audit was repeated during the `v3.0.0-rc.1`
  fix-forward pass.
- No surviving reportable security findings remain after the fixes in commit
  `f5179df`.
- The fixes harden CLI diagnostics, APRS-IS login field validation, and TCP
  default timeout behavior.
- Verification passed with formatting, focused tests, full workspace tests,
  clippy with warnings denied, and docs verification.
- Merge should proceed only through PR checks and the normal release gates.

## Closed Findings

- CLI diagnostic control-character injection: user-controlled option, fail-on,
  and path values are now escaped before display.
- APRS-IS login/profile ASCII-control injection: login fields now reject all
  ASCII control bytes, not only CR and LF.
- TCP default indefinite blocking: default TCP connect and read options now use
  finite timeouts; callers can explicitly opt into blocking behavior by setting
  timeout fields to `None`.

## Verification Evidence

- `cargo fmt --check`
- `cargo test -p aprs-cli -p aprs-transport-aprs-is`
- `cargo test -p aprs-transport-tcp -p libaprs-engine`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `scripts/verify-docs.sh`

All commands passed. `scripts/verify-docs.sh` also passed after this summary was
added and linked.

## PR Task List

- [x] Add tracked security-audit summary for release evidence.
- [ ] Push `codex/repo-security-audit-fix-forward`.
- [ ] Open PR to `main`.
- [ ] Wait for CI, security, merge-gate, and supply-chain checks.
- [ ] Fix forward on the same branch if any check fails.
- [ ] Merge only when all checks and secure-review findings are clean.
- [ ] Rerun release verification from fresh `main` after merge.
