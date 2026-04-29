# Codex Project Instructions

## Communication

- Use BLUF format.
- Use concise bullets.
- Prioritise the top five most important items.

## Project Rules

- Protocol-first.
- Preserve raw bytes.
- Fail closed on malformed packets.
- Treat packet and transport input as untrusted.
- Follow secure coding standards and OWASP-aligned input-handling guidance.

## Workflow

- Use feature branches with the `codex/` prefix.
- Use `CARGO_HOME=/tmp/libaprs-cargo-home` for Cargo commands that write
  registry, advisory, package, or cache state.
- Prefer `scripts/verify-release.sh` for full local release verification.
- Do not commit Cargo credentials, registry caches, package caches, advisory
  databases, or temporary Cargo homes.

## Secure Review And Release Gates

- Before pushing or merging to `main`, perform a secure code review.
- Do not merge to `main` if secure review findings remain open.
- Propose fixes for review findings before asking for next-step decisions.
- Do not publish crates before secure review, local release gates, remote CI,
  security gates, and GitHub Release evidence are clean.
- Use `scripts/publish-release.sh` for crates.io publication so GitHub Release
  publication and latest-release verification are enforced.
