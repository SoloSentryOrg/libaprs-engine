# Supply Chain Evidence

## BLUF

- The core parser path remains dependency-light and network-free.
- New roadmap task 2-5 APIs add no third-party dependencies.
- Release verification uses pinned release/security tooling where CI installs
  tools.
- Crates.io publication remains guarded by secure review, local gates, remote
  CI, security gates, GitHub Release evidence, and post-publication smoke.
- Generated SBOM or binary attestation artifacts are application/release
  evidence, not parser runtime behavior.

## Dependency Controls

The repository uses:

- `cargo metadata` to validate workspace metadata,
- `cargo audit` for RustSec advisories when installed,
- `cargo deny check` for advisory, license, duplicate, source, and wildcard
  dependency policy when installed,
- `scripts/install-release-tools.sh` to install pinned release/security tools
  in CI,
- `scripts/verify-release.sh` as the local release gate, and
- `scripts/publish-release.sh` as the guarded crates.io and GitHub Release
  publication path.

## SBOM Guidance

For releases or downstream deployments that require an SBOM:

- Generate SBOMs from the exact release commit after local and remote gates
  pass.
- Keep SBOM artifacts outside the source tree unless they are intentionally
  checked in as release evidence.
- Do not commit Cargo registry caches, advisory databases, temporary Cargo
  homes, crates.io credentials, or generated binary artifacts.
- Record SBOM generation commands and artifact locations in `docs/release.md`
  when an SBOM is part of release evidence.

## Current Evidence

The roadmap task 2-5 changes are standard-library only for the new core helper
APIs. Transport examples use existing workspace crates and do not introduce
additional runtime ownership into `libaprs-engine`.
