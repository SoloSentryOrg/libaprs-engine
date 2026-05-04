# v3.0.0 Breaking-Change Decision Record

## BLUF

- No `v3.0.0` public API breaking change is approved for `v3.0.0-rc.1`.
- `v3.0.0-rc.1` is approved as a no-intentional-break major release candidate
  after the `v2.6.0` evidence-first track.
- Encoder, service-toolkit, and APRS-IS profile helpers are additive modules,
  not breaking API replacements.
- Any future break must have a concrete downstream report, migration path,
  compatibility test update, release-note entry, and secure review.
- If semver checks, secure review, or downstream smoke reveal a real break,
  fix it forward or document the intentional break before publication.

## Current Candidate Areas

| Candidate | Status | Rationale |
| --- | --- | --- |
| Split codec, semantic, policy, diagnostic, and encoder modules more aggressively | Not approved for `v3.0.0-rc.1` | Additive modules are working; no downstream report shows current grouping causes unsafe use. |
| Replace broad semantic enum variants with narrower typed views | Not approved for `v3.0.0-rc.1` | `AprsData` still provides useful byte-preserving access and can grow additively. |
| Introduce stronger transport receive-loop traits | Not approved for `v3.0.0-rc.1` | Existing adapter-specific helpers keep runtime and network ownership with callers. |
| Rename diagnostic or policy codes | Not approved for `v3.0.0-rc.1` | Stable codes are already used in docs and tests; no ambiguity report exists. |
| Change feature organization | Not approved for `v3.0.0-rc.1` | Current optional features remain small and documented. |

## Evidence Required Before Approval

Before any candidate can move to approved:

- record the downstream integration and crate version affected,
- describe the unsafe, confusing, or unmaintainable behavior,
- prove an additive fix is not enough,
- add or update compatibility and migration tests,
- document the old-to-new API mapping, and
- record secure-review and release-gate implications.

## Current Decision

Prepare `v3.0.0-rc.1` as a no-intentional-break release candidate. The release
candidate validates version metadata, package publication, migration guidance,
downstream smoke, SBOM/hash evidence, and remote gates for the next major line.
No public API removal, rename, or semantic behavior break is approved in this
record.
