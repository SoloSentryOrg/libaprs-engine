---
name: Bug report
about: Report a parser, policy, CLI, or transport bug
title: "bug: "
labels: bug
---

## Summary

Describe the failure and expected behavior.

## Packet Input

Provide a minimized packet only if it is safe to publish. Redact private
callsigns, precise private locations, operator names, and credentials.

```text

```

## Impact

- Parser panic:
- Raw-byte preservation issue:
- Fail-closed issue:
- Policy/transport issue:

## Verification

List commands run, such as:

```sh
cargo test -p libaprs-engine
cargo clippy --all-targets --all-features -- -D warnings
```
