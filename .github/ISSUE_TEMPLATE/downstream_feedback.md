---
name: Downstream feedback
about: Report API, migration, or integration feedback for release planning
title: "downstream: "
labels: downstream
---

## Integration Type

Parser library, CLI, service ingestion, transport adapter, release tooling, or
another downstream use case.

## Crate And Version

- Crate:
- Version:
- Feature flags:

## Packet Or Transport Boundary

Describe where bytes enter your integration and where framing ends. Provide a
minimized packet only if it is safe to publish.

```text

```

## Raw-Byte And Fail-Closed Behavior

- Were accepted raw bytes preserved exactly?
- Did malformed input fail closed?
- Did invalid UTF-8 remain byte-preserving?
- Did the integration need a bounded read or packet limit?

## API Or Migration Pain

Name the API, diagnostic code, transport helper, schema, or behavior that caused
confusion or made integration harder.

## Proposed Outcome

- Additive `1.x` fix:
- Documentation or example update:
- Possible `v2.0.0` breaking change:

## Reproduction Or Verification

List commands, fixtures, or application checks that demonstrate the issue.
