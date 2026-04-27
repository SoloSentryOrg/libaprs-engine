# Publishing

This repository is ready for crates.io package validation, but publishing
requires a crates.io account token and should be done only from a clean,
verified release commit.

## Crates

Publish crates in dependency order:

1. `libaprs-engine`
2. `aprs-transport-file`
3. `aprs-transport-tcp`
4. `aprs-transport-aprs-is`
5. `aprs-transport-async`
6. `aprs-transport-ax25`
7. `aprs-transport-channel`
8. `aprs-transport-corpus`
9. `aprs-transport-file-watch`
10. `aprs-transport-http`
11. `aprs-transport-kiss`
12. `aprs-transport-mqtt`
13. `aprs-transport-serial`
14. `aprs-transport-udp`
15. `aprs-cli`

The adapter and CLI crates use versioned path dependencies:

```toml
libaprs-engine = { version = "0.5.0", path = "../libaprs-engine" }
```

Cargo uses the local path in this workspace and the version requirement when
packaging for crates.io.

## Dry Run

Run package validation before publishing the core crate:

```sh
cargo package -p libaprs-engine
```

Before `libaprs-engine` exists on crates.io, Cargo cannot package dependent
crates because their registry dependency is not resolvable yet. After
`libaprs-engine` is published and visible in the crates.io index, validate the
dependent crates:

```sh
cargo package -p aprs-transport-file
cargo package -p aprs-transport-tcp
cargo package -p aprs-transport-aprs-is
cargo package -p aprs-transport-async
cargo package -p aprs-transport-ax25
cargo package -p aprs-transport-channel
cargo package -p aprs-transport-corpus
cargo package -p aprs-transport-file-watch
cargo package -p aprs-transport-http
cargo package -p aprs-transport-kiss
cargo package -p aprs-transport-mqtt
cargo package -p aprs-transport-serial
cargo package -p aprs-transport-udp
cargo package -p aprs-cli
```

## Publish

Authenticate with crates.io outside the repository:

```sh
cargo login
```

In restricted environments where `~/.cargo` is not writable, keep Cargo state
outside the repository:

```sh
mkdir -p /tmp/libaprs-cargo-home
cp "$HOME/.cargo/credentials.toml" /tmp/libaprs-cargo-home/credentials.toml
CARGO_HOME=/tmp/libaprs-cargo-home cargo publish -p libaprs-engine
```

Do not commit Cargo credentials, registry caches, package caches, advisory
databases, or temporary Cargo homes.

Publish in dependency order:

```sh
cargo publish -p libaprs-engine
cargo publish -p aprs-transport-file
cargo publish -p aprs-transport-tcp
cargo publish -p aprs-transport-aprs-is
cargo publish -p aprs-transport-async
cargo publish -p aprs-transport-ax25
cargo publish -p aprs-transport-channel
cargo publish -p aprs-transport-corpus
cargo publish -p aprs-transport-file-watch
cargo publish -p aprs-transport-http
cargo publish -p aprs-transport-kiss
cargo publish -p aprs-transport-mqtt
cargo publish -p aprs-transport-serial
cargo publish -p aprs-transport-udp
cargo publish -p aprs-cli
```

Wait for each published dependency to become available before publishing crates
that depend on it.

## Release Requirements

- Run the local verification gate in `docs/verification.md`.
- Confirm GitHub Actions passed on the release commit.
- Run `cargo package -p libaprs-engine`.
- After `libaprs-engine` is published, run package validation for dependent
  crates before publishing them.
- Confirm `CHANGELOG.md` describes the release.
- Tag only after package validation and CI both pass.
