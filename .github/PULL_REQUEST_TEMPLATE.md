## BLUF

- What changed:
- Why it changed:
- User or developer impact:
- Release impact:
- Security impact:

## Verification

- [ ] `scripts/verify-docs.sh`
- [ ] `cargo test --all-features`
- [ ] `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] Secure code review completed with no open findings.

## Security Checklist

- [ ] Untrusted input remains bounded and validated.
- [ ] Raw packet bytes remain preserved for accepted packets.
- [ ] Malformed packet behavior remains fail closed.
- [ ] No credentials, private packet data, or local-only artifacts are included.
- [ ] Dependency, workflow, or release changes are explained.

## Notes

Link related issues, roadmap items, release evidence, or follow-up work.
