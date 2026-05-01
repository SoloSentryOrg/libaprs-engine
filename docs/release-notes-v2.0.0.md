# libaprs-engine v2.0.0

`v2.0.0` promotes the tested `v2.0.0-rc.2` release candidate to the final major
release.

## Highlights

- Finalizes the narrow `v2.0.0` API change: the library-level
  `ParsedPacket::to_json()` diagnostic convenience method was removed in the
  release-candidate line.
- Keeps CLI JSON as CLI-owned diagnostic output with an explicit
  `schema_version`.
- Keeps raw-byte preservation, fail-closed malformed-packet behavior, and the
  network-free parser core unchanged from the release-candidate line.
- Publishes the core crate, optional transport crates, and CLI at `2.0.0`.

## Migration

Use `ParsedPacket::to_diagnostic()` with the `serde` feature,
`serde_support::PacketDiagnostic`, `PacketSummary`, `EngineEvent`, CLI `--json`,
or an application-owned schema instead of the removed library `to_json()` helper.

See `docs/v2-migration.md` and `docs/v2-breaking-changes.md` for the full
decision record and migration guidance.
