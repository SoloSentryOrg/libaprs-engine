# Contributing

This project is protocol-first: preserve raw packet bytes, fail closed at codec
boundaries, and keep network and transport behavior outside the parser core.

## Development Rules

- Treat all packet input as untrusted bytes.
- Add regression tests before parser, semantic, policy, or transport changes.
- Do not convert packet bytes to `String` before parsing.
- Preserve accepted packet bytes exactly.
- Return explicit errors or malformed semantic variants instead of panicking on
  untrusted input.
- Keep dependencies minimal and justify any new dependency in the pull request.

## Local Verification

Run the release gate before submitting changes:

```sh
scripts/verify-release.sh
```

Use the default Cargo home for normal local development. When the default Cargo
home is not writable, use a temporary Cargo home:

```sh
mkdir -p /tmp/libaprs-cargo-home
CARGO_HOME=/tmp/libaprs-cargo-home scripts/verify-release.sh
```

For focused parser work, also run:

```sh
cargo test -p libaprs-engine
cargo test --examples
cargo clippy --all-targets --all-features -- -D warnings
```

## Parser And Semantic Changes

- Add accepted fixtures for newly supported packet families.
- Add malformed fixtures for fail-closed or malformed-semantic behavior.
- Add policy-rejection tests when policy behavior changes.
- Add fuzz corpus seeds only when they are safe to publish.
- Keep private callsigns, operator names, and precise private locations out of
  checked-in fixtures.

## Public API Changes

- Update `docs/stability.md` when stable-intent APIs change.
- Update `docs/api.md` and `docs/examples.md` for developer-facing APIs.
- Run `cargo semver-checks check-release -p libaprs-engine`.
- Document breaking pre-1.0 changes in `CHANGELOG.md`.

## Secure Review Checklist

- Input validation remains fail-closed.
- Raw bytes remain available for accepted packets.
- Counters and diagnostics do not wrap or hide failures.
- Errors do not expose secrets, credentials, or private packet payloads by
  default.
- Transport examples bound input and keep authentication, TLS, retries, and
  timeouts application-owned.
