# Public Repository Readiness

![libaprs-engine documentation header](assets/brand/docs-header.svg)

## BLUF

- Run a full secret-history scan before changing repository visibility.
- Keep security reports private; do not use public issues for vulnerabilities.
- Keep `main` protected with pull requests and required status checks.
- Do not require an approving review for a solo-maintainer repo unless a second
  reviewer is available.
- Disable blank issues so public intake uses safe templates.
- Set repository topics, homepage, and social preview before launch.

## Gitleaks

Gitleaks is an open-source secret scanner. It scans the working tree and git
history for tokens, credentials, private keys, and other secret-like material
that should not become public.

Install locally on macOS:

```sh
brew install gitleaks
```

Alternative installs:

```sh
docker run --rm -v "$PWD:/repo" zricethezav/gitleaks:latest detect --source /repo --redact
go install github.com/gitleaks/gitleaks/v8@latest
```

Run before public visibility changes:

```sh
scripts/check-secrets.sh
```

If findings are real, rotate the exposed secret first, then remove or rewrite
the leaked material before making the repository public. Rewriting public git
history is disruptive, so complete this scan while the repository is still
private.

## Public Launch Checklist

- [ ] `gitleaks detect --redact --source .` completes with no unresolved
      findings.
- [ ] `scripts/verify-release.sh` completes, or skipped optional gates are
      explicitly documented.
- [ ] Remote CI and security workflows are green on `main`.
- [ ] Branch protection requires pull requests and `Merge Gate`, but does not
      require an approving review unless a second reviewer can approve PRs.
- [ ] `SECURITY.md`, `SUPPORT.md`, `CODE_OF_CONDUCT.md`,
      `.github/CODEOWNERS`, and `.github/PULL_REQUEST_TEMPLATE.md` exist.
- [ ] GitHub private vulnerability reporting is enabled after the repository is
      public.
- [ ] Blank issues are disabled.
- [ ] Repository topics, description, homepage, and social preview are set.
- [ ] Latest GitHub Release and crates.io versions match.
- [ ] No generated caches, build output, private fixtures, credentials, or local
      machine-specific files are tracked.

## Manual GitHub Steps

Some repository presentation settings may require the GitHub web UI:

1. Open repository settings.
2. Upload `docs/assets/brand/social-preview.png` as the social preview image.
3. Confirm private vulnerability reporting is enabled after the repository is
   public.
4. Change repository visibility to public only after the checklist is complete.
