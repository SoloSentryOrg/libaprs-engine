# Security Policy

## BLUF

- Treat packet, transport, CLI, and fixture input as untrusted bytes.
- Do not report vulnerabilities through public issues.
- Use GitHub private vulnerability reporting when available.
- Provide minimized reproduction data that is safe to publish or privately
  disclose.
- Do not include credentials, private packets, operator names, or precise
  private locations in reports.

## Supported Versions

Security fixes are prioritized for the current published major release line.
Older major versions may receive fixes when the impact is severe and the patch
is low risk.

| Version | Supported |
| --- | --- |
| `2.x` | Yes |
| `< 2.0.0` | No |

## Reporting A Vulnerability

Use GitHub private vulnerability reporting for this repository when it is
available. If private reporting is not visible, open a minimal public issue that
asks for a private maintainer contact path without disclosing technical details.

Include:

- A short description of the vulnerability class.
- A minimized packet, frame, or input only when it is safe to share.
- The affected crate, version, feature flags, and operating mode.
- The expected secure behavior and the observed behavior.
- Any local verification commands that reproduce the issue.

Do not include:

- Credentials, API tokens, passwords, or private keys.
- Private station packets, operator names, or precise private locations.
- Full production logs.
- Exploit instructions beyond what is required to reproduce the issue.

## Security Scope

In scope:

- Parser panics or fail-open malformed packet handling.
- Raw-byte preservation regressions.
- Unbounded reads or avoidable resource exhaustion.
- Unsafe disclosure in diagnostics or CLI output.
- Transport helper behavior that violates documented byte-preserving
  boundaries.
- Supply-chain or release-process vulnerabilities.

Out of scope:

- Vulnerabilities in applications that embed these crates incorrectly.
- Network authentication, TLS, retry, and deployment policy choices owned by a
  downstream application.
- Reports that require publishing private packet data.

## Disclosure Process

Maintainers will triage reports as capacity allows, keep discussion private
until a fix or mitigation is available, and publish release notes when a
security-relevant fix ships.
