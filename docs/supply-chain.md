# Supply Chain Evidence

![libaprs-engine documentation header](assets/brand/docs-header.svg)

## BLUF

- The core parser path remains dependency-light and network-free.
- Tracked CycloneDX SBOMs live under `supply-chain/sbom/`.
- Tracked repository integrity hashes live in `supply-chain/SHA256SUMS`.
- Rust source files are identified by Git commit and tag, not duplicated in
  `SHA256SUMS`.
- Release verification fails if tracked SBOM or hash evidence drifts.
- Crates.io publication remains guarded by secure review, local gates, remote
  CI, security gates, GitHub Release evidence, and post-publication smoke.
- Generated SBOM and hash artifacts are release evidence, not parser runtime
  behavior.

## Dependency Controls

The repository uses:

- `cargo metadata` to validate workspace metadata,
- `cargo audit` for RustSec advisories when installed,
- `cargo deny check` for advisory, license, duplicate, source, and wildcard
  dependency policy when installed,
- `cargo cyclonedx` for per-crate CycloneDX SBOM generation,
- `scripts/generate-supply-chain-evidence.sh` to regenerate SBOM and hash
  evidence,
- `scripts/test-supply-chain-evidence.sh` to fail closed on stale evidence,
- `scripts/install-release-tools.sh` to install pinned release/security tools
  in CI,
- `scripts/verify-release.sh` as the local release gate, and
- `scripts/publish-release.sh` as the guarded crates.io and GitHub Release
  publication path.

## Tracked Evidence

The tracked supply-chain evidence is:

- `supply-chain/sbom/*.cdx.json`: deterministic CycloneDX 1.5 SBOMs for each
  workspace crate, generated with all features and all target dependencies.
- `supply-chain/SHA256SUMS`: SHA-256 hashes for supply-chain control files,
  dependency manifests, lockfiles, dependency-management config, workflows,
  scripts, policy files, supply-chain documentation, and tracked SBOMs.

Regenerate and check evidence with:

```bash
scripts/generate-supply-chain-evidence.sh
scripts/test-supply-chain-evidence.sh
```

CI runs the same guard in the `Supply Chain` workflow. The merge gate requires
that check when dependency manifests, lockfiles, supply-chain evidence, supply
chain documentation, release scripts, or supply-chain workflows change.

## Source Integrity

Git is the source integrity control for Rust source files:

- Record the exact commit SHA and release tag in release evidence.
- Prefer signed tags and signed release artifacts where practical.
- Do not list every `*.rs` file in `supply-chain/SHA256SUMS`; that duplicates
  Git, creates churn, and does not add meaningful tamper resistance for a
  tracked repository file.
- Hash source archives, crate packages, binaries, or other release artifacts
  when they are distributed outside the Git object model.

## Release Artifact Hashes

For releases or downstream deployments that distribute artifacts:

- Generate SBOMs and hashes from the exact release commit after local and
  remote gates pass.
- Hash `.crate`, source archive, binary, and SBOM artifacts that are published
  as release assets or handed to downstream consumers.
- Record artifact hash locations in `docs/release.md` for the release.
- Do not commit Cargo registry caches, advisory databases, temporary Cargo
  homes, crates.io credentials, or generated binary artifacts.
- Keep release-package hashes in release evidence or GitHub Release assets, not
  in the main-branch repository manifest unless the artifact is also tracked.

## Current Evidence

Main branch carries deterministic per-crate SBOMs and SHA-256 evidence for the
tracked supply-chain control surface. Runtime parser behavior remains
independent from generated evidence files.
