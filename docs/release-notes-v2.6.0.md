# libaprs-engine v2.6.0

## BLUF

- `v2.6.0` is an additive evidence-first readiness release before any
  `v3.0.0` release-candidate work.
- No `v3.0.0` breaking change is approved.
- The downstream feedback log is internal-only and is no longer exposed through
  public README navigation or public GitHub issue templates.
- Docs gates now keep internal evidence handling, the `v2.6.0` roadmap entry,
  release notes, and the gated `v3.0.0` decision record in sync.
- Security regression coverage expands around untrusted input, bounded
  transport handling, raw-byte preservation, and fail-closed malformed packet
  behavior.

## Changes

- Added `scripts/check-internal-docs.sh` and wired it into docs verification so
  public-facing docs cannot accidentally link to the internal downstream
  evidence log.
- Added `scripts/check-v2-6-evidence.sh` and wired it into docs verification so
  the evidence-first milestone, release notes, internal evidence marker, and
  gated `v3.0.0` decision stay explicit.
- Removed the public downstream-feedback issue template.
- Expanded abuse-resistance tests around oversized packets, invalid UTF-8,
  malformed semantic payloads, nested third-party packet handling, and bounded
  malformed-event evidence.
- Updated current package examples to `2.6.0`.

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

## Upgrade Notes

Existing `2.5.0` users can update dependency versions to `2.6.0`. This release
does not intentionally break public APIs.
