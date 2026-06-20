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
- Before planning organization-spanning workflow, security-control, package, project, or governance changes, review the central SoloSentry lessons register in `SoloSentryOrg/github-enterprise-management-solosentry` at `docs/lessons-learned/register.md`.
- For organization-spanning changes, carry applicable lessons into the plan and include a short `complexity removed or justified` statement.
- Stop repeated same-cluster fix-forward until root cause is recorded in the relevant issue, PR, or central lessons register.
- Treat this as a solo-maintainer repository unless the user says otherwise.
- Do not recommend or apply branch-protection rules that require an
  independent approval when no second reviewer has been configured; that blocks
  solo-maintainer merges. Prefer PR-required plus `Merge Gate`-required rules.
- Use the default Cargo home for normal local verification, packaging, audit,
  deny, semver, and publish commands.
- Use `CARGO_HOME=/tmp/libaprs-cargo-home` only when the default `~/.cargo`
  path is not writable or Cargo registry, advisory, package, or cache writes
  fail.
- Prefer `scripts/verify-release.sh` for full local release verification.
- Do not commit Cargo credentials, registry caches, package caches, advisory
  databases, or temporary Cargo homes.

## Sub-Agent Use

- Sub-agents are allowed when they provide clear benefit, such as focused
  review, independent investigation, parallel checks, or bounded
  implementation.
- Use sub-agents only with explicit ownership, narrow scope, and clear expected
  outputs.
- Do not use sub-agents for tightly coupled edits, urgent blocking work,
  ambiguous tasks, or changes likely to create merge conflicts.
- For code changes, assign disjoint write scopes and require changed file paths
  in the sub-agent summary.
- Verify sub-agent findings and changes before relying on them, committing, or
  merging.

## Secure Review And Release Gates

- Before pushing or merging to `main`, perform a secure code review.
- Do not merge to `main` if secure review findings remain open.
- Propose fixes for review findings before asking for next-step decisions.
- Do not publish crates before secure review, local release gates, remote CI,
  security gates, and GitHub Release evidence are clean.
- Use `scripts/publish-release.sh` for crates.io publication so GitHub Release
  publication and latest-release verification are enforced.
- After each release, update GitHub Project #3 as the primary roadmap store and
  update `ROADMAP.md` as the repository backup before treating the release as
  closed.
