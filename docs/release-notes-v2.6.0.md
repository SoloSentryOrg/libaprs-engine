# Release Notes v2.6.0

## BLUF

- `v2.6.0` is planned as an additive evidence-first readiness release before
  any `v3.0.0` release-candidate work.
- No `v3.0.0` breaking change is approved by this release plan.
- The downstream feedback log is internal only and is no longer exposed through
  public README navigation or public GitHub issue templates.
- New docs gates keep internal evidence handling, the `v2.6.0` roadmap entry,
  and the gated `v3.0.0` decision record in sync.
- Security regression coverage focuses on untrusted input, bounded transport
  handling, raw-byte preservation, and fail-closed malformed packet behavior.

## Planned Changes

- Add `scripts/check-internal-docs.sh` and wire it into docs verification so
  public-facing docs cannot accidentally link to the internal downstream
  evidence log.
- Add `scripts/check-v2-6-evidence.sh` and wire it into docs verification so
  the evidence-first milestone, release notes, internal evidence marker, and
  gated `v3.0.0` decision stay explicit.
- Remove the public downstream-feedback issue template.
- Expand abuse-resistance tests around oversized packets, invalid UTF-8,
  malformed semantic payloads, and nested third-party packet handling.

## Security Notes

- Packet and transport input remains untrusted.
- Accepted packets must preserve raw bytes exactly.
- Malformed packet shape must fail closed.
- Internal evidence may include integration details that should not be promoted
  into public issue templates without review and sanitization.

## Release Gates

- `scripts/verify-docs.sh`
- `cargo test -p libaprs-engine --test engine`
- `cargo test -p libaprs-engine --test codec`
- `scripts/verify-release.sh`
