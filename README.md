# Monadnock Cyber GPG

A modern, open-source reincarnation of classic **PGP Desktop** — keys, file & data
encryption, email, an encrypted vault, and (later) secure messaging and voice — in
a beautiful, easy-to-use app that lives in your **menu bar / system tray** and
interoperates with existing **GPG/OpenPGP** keys.

> **Status: Phase 1 in progress.** A polished React + Tauri desktop app (sidebar
> nav: Keys · Encrypt · Decrypt) with a persistent keyring — generate, import,
> export, and delete OpenPGP keys, and encrypt/decrypt text — all through the
> [`mc-core`](mc-core) Sequoia-PGP crypto core. See the
> [requirements spec](REQUIREMENTS.md) for the full roadmap.

## Vision

OpenPGP is excellent but its tooling is intimidating. Monadnock Cyber GPG brings
the old integrated-suite experience to a modern audience with a genuinely friendly
interface — without compromising on real, standards-based cryptography.

- **Cross-platform:** Windows + macOS first, then iOS/Android. One Rust core, one UI.
- **Standards-based:** full OpenPGP via [Sequoia-PGP](https://sequoia-pgp.org); works
  with everyone already using GPG.
- **Private by design:** local-first, zero telemetry, no key escrow.
- **Trustworthy releases:** reproducible builds, signed + checksummed downloads.

## Planned capabilities

| Module | Phase |
|---|---|
| Key management (create/import/verify/hardware tokens) | v1 |
| File & folder encryption / signing | v1 |
| Text & clipboard encryption | v1 |
| Menu-bar / system-tray app with quick actions | v1 |
| Email encryption (standalone + plug-ins) | v2 |
| Encrypted vault | v2 |
| Mobile (iOS/Android) | v3 |
| Secure messaging (MLS/Signal) | v4 |
| Voice / video | v5 |

See **[REQUIREMENTS.md](REQUIREMENTS.md)** for the full spec, threat model, and roadmap.

## Tech stack

- **Core:** Rust + [`sequoia-openpgp`](https://crates.io/crates/sequoia-openpgp)
- **UI:** [Tauri 2](https://tauri.app) + React + Tailwind (shadcn-flavored components)

## Build & run

Requires [Rust](https://rustup.rs) and [Node](https://nodejs.org).

```sh
# 1. build the UI (one time, and after any UI change)
cd app
npm install
npm run build

# 2. build & launch the app
cd src-tauri
cargo run
```

It opens as a **desktop window** (and a padlock in the **menu bar / system
tray**): create/import keys, then encrypt and decrypt. Keys persist under your
app-data directory; nothing leaves the machine.

Run the crypto core's tests with `cd mc-core && cargo test`.

> Dev note: the app loads its **embedded** built UI (`app/dist`), so rebuild the
> frontend (`npm run build`) after UI changes, then `cargo run`.

## Repository layout

```
mc-core/        Rust crypto core (Sequoia-PGP): keyring, encrypt, decrypt
app/
  src/          React + Tailwind UI (sidebar: Keys / Encrypt / Decrypt)
  src-tauri/    Tauri 2 shell — commands, menu-bar/tray, desktop window
  icon/         icon generator + source PNG
  dist/         built UI (generated; embedded into the app)
REQUIREMENTS.md Full spec & roadmap
```

## Contributing

Early days — issues and ideas welcome. The project intends to keep all
security-critical logic in one audited Rust core and never roll its own crypto.

## License

[Apache-2.0](LICENSE) — © 2026 Monadnock Cyber
