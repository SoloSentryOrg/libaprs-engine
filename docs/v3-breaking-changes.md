# v3.0.0 Breaking-Change Decision Record

## BLUF

- No `v3.0.0` breaking change is approved yet.
- Continue additive `2.x` development while downstream evidence is still
  insufficient for a major break.
- Encoder, service-toolkit, and APRS-IS profile helpers are additive modules,
  not breaking API replacements.
- Any future break must have a concrete downstream report, migration path,
  compatibility test update, release-note entry, and secure review.
- `v3.0.0-rc.1` should not start until `v2.5.0` records specific evidence.

## Current Candidate Areas

| Candidate | Status | Rationale |
| --- | --- | --- |
| Split codec, semantic, policy, diagnostic, and encoder modules more aggressively | Not approved | Additive modules are working; no downstream report shows current grouping causes unsafe use. |
| Replace broad semantic enum variants with narrower typed views | Not approved | `AprsData` still provides useful byte-preserving access and can grow additively. |
| Introduce stronger transport receive-loop traits | Not approved | Existing adapter-specific helpers keep runtime and network ownership with callers. |
| Rename diagnostic or policy codes | Not approved | Stable codes are already used in docs and tests; no ambiguity report exists. |
| Change feature organization | Not approved | Current optional features remain small and documented. |

## Evidence Required Before Approval

Before any candidate can move to approved:

- record the downstream integration and crate version affected,
- describe the unsafe, confusing, or unmaintainable behavior,
- prove an additive fix is not enough,
- add or update compatibility and migration tests,
- document the old-to-new API mapping, and
- record secure-review and release-gate implications.

## Current Decision

Continue on the additive `2.x` track through `v2.5.0`. Prepare `v3.0.0-rc.1`
only if this record changes from evidence, not preference.
