# Supply Chain Evidence

## BLUF

- `sbom/*.cdx.json` contains deterministic CycloneDX SBOMs for each workspace
  crate.
- `SHA256SUMS` hashes dependency manifests, lockfiles, dependency-management
  config, workflows, scripts, policy, supply-chain docs, and tracked SBOMs.
- Rust source files are intentionally not listed in `SHA256SUMS`; source
  identity is the Git commit and release tag.
- Regenerate with `scripts/generate-supply-chain-evidence.sh`.
- Verify with `scripts/test-supply-chain-evidence.sh`.
