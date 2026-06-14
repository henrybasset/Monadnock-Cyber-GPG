# Monadnock Cyber GPG

A modern, open-source reincarnation of classic **PGP Desktop** — keys, file & data
encryption, email, an encrypted vault, and (later) secure messaging and voice — in
a beautiful, easy-to-use app that lives in your **menu bar / system tray** and
interoperates with existing **GPG/OpenPGP** keys.

> **Status: Phase 0 works.** A menu-bar app (Tauri 2 + Rust) generates an OpenPGP
> key and encrypts/decrypts text through the [`mc-core`](mc-core) Sequoia-PGP
> crypto core. See the [requirements spec](REQUIREMENTS.md) for the full roadmap.

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
- **UI:** [Tauri 2](https://tauri.app) + React + Tailwind + shadcn/ui

## Build & run (Phase 0)

Requires [Rust](https://rustup.rs) and Node. From the repo root:

```sh
# run the crypto core's round-trip test
cd mc-core && cargo test

# build & launch the menu-bar app
cd ../app/src-tauri && cargo run
```

The app appears in the **menu bar / system tray** (a padlock icon). The window
lets you generate a key and encrypt/decrypt text — proving the Tauri ⇆ Rust ⇆
Sequoia stack end to end.

## Repository layout

```
mc-core/        Rust crypto core (Sequoia-PGP): generate_key, encrypt, decrypt
app/
  ui/           Phase 0 frontend (static HTML/JS; React+Tailwind lands in Phase 1)
  src-tauri/    Tauri 2 shell — commands, menu-bar/tray, window
  icon/         icon generator + source PNG
REQUIREMENTS.md Full spec & roadmap
```

> Phase 0 uses a minimal static UI to prove the stack; the React + Tailwind +
> shadcn/ui interface from the spec arrives in Phase 1.

## Contributing

Early days — issues and ideas welcome. The project intends to keep all
security-critical logic in one audited Rust core and never roll its own crypto.

## License

[Apache-2.0](LICENSE) — © 2026 Monadnock Cyber
