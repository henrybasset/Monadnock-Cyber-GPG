# Monadnock Cyber GPG — Requirements & Architecture

A modern, open-source reincarnation of classic **PGP Desktop**: keys, file/data
encryption, email, an encrypted vault, and (later) secure messaging and voice —
wrapped in a beautiful, genuinely easy-to-use interface, and fully interoperable
with existing GPG/OpenPGP keys.

## Decisions
- **Scope:** the full suite, delivered as a phased roadmap (not all in v1).
- **Platforms:** Desktop first (**Windows + macOS**); iOS/Android (and possibly web) later.
- **Crypto base:** **Full OpenPGP**, GPG-interoperable, built on **Sequoia-PGP**.
- **License:** Apache-2.0. **UI:** React + Tailwind + shadcn/ui on **Tauri 2**.

## Principles
1. **Never roll our own crypto.** All OpenPGP via the vetted **Sequoia-PGP** (Rust)
   engine; hardware-token crypto via OpenPGP-card / PKCS#11.
2. **PGP is great for data-at-rest, wrong for chat.** OpenPGP has no forward
   secrecy, so the messaging/voice modules use a modern ratchet —
   **MLS (RFC 9420)** or the **Signal protocol** — not PGP. OpenPGP identities can
   bootstrap trust; the live session crypto is separate.
3. **Phase ruthlessly.** Keys+files+text is achievable; email is fiddly;
   vault/disk is OS-specific; voice ≈ building Signal (servers/accounts/push) and
   is the largest, latest piece.
4. **Trust is the product.** Reproducible builds, signed + checksummed releases,
   published SBOM, zero telemetry, local-first, open governance.

## Architecture
- **Shared Rust core (`mc-core`):** all crypto/key/state logic via
  `sequoia-openpgp`, reused by every platform — security-critical code in one place.
- **UI shell: Tauri 2** (Rust backend + web frontend; **React + Tailwind +
  shadcn/ui**). Tauri 2 targets Windows, macOS, Linux, iOS, and Android; its
  tray/menu-bar APIs cover the macOS menu bar and the Windows system tray.
- **Key storage:** OS-native secure stores (macOS Keychain/Secure Enclave,
  Windows DPAPI/Credential Manager, iOS Secure Enclave, Android Keystore) plus
  **hardware tokens** (YubiKey / OpenPGP card / FIDO2).

## Modules
Tags: **[P1]** v1 desktop · **[P2]** fast-follow · **[P3]** later · **[P4]** largest/last.

- **A. Key management [P1]** — create (RSA-4096 / Ed25519), import, export, revoke,
  expiry; friendly keyring (identities, fingerprints, verification status);
  keyserver/WKD; QR fingerprint verify; backup/restore; hardware tokens.
- **B. File & folder encryption/signing [P1]** — right-click / drag-drop encrypt,
  decrypt, sign, verify (Finder/Explorer); multi-recipient; detached signatures;
  secure shredding; encrypted archives (PGP Zip equivalent).
- **C. Text & clipboard encryption [P1]** — encrypt/decrypt/sign snippets and the
  clipboard; clean compose/read window with recipient picker.
- **H. Menu-bar / system-tray presence [P1]** — primary, always-available entry
  point in the **macOS menu bar** and **Windows system tray**, with a full main
  window behind it. Quick actions: encrypt/decrypt clipboard, sign, lock/unlock
  vault, recent items, new message, open window, lock, quit. Launch-at-login;
  optional global hotkey; lightweight background presence.
- **D. Email encryption [P2]** — standalone compose/read; plus Apple Mail / Outlook
  plug-ins or an SMTP/IMAP companion (per-client integration is the fiddly part).
- **E. Encrypted vault / disk [P2/P3]** — a transparent encrypted folder
  (Cryptomator-style per-file) mounted as a virtual drive [P2]; manage/wrap OS FDE
  (FileVault/BitLocker) [P3]. No custom disk driver.
- **F. Secure messaging [P3/P4]** — Signal-style E2EE chat (MLS or Signal protocol,
  forward secrecy); identity bootstrapped from OpenPGP keys; backend + accounts +
  push. Backend stance TBD (Matrix-federated vs Monadnock relay vs P2P).
- **G. Voice / video [P4]** — E2EE calls via WebRTC (DTLS-SRTP); needs TURN/SFU.

## Non-functional requirements
- **Security:** memory zeroization, no plaintext spill, OS keystore + hardware
  tokens, optional passphrase/biometric unlock, documented threat model.
- **Usability:** friendly defaults, progressive disclosure, plain-language
  onboarding (what a key is, why verify), dark/light.
- **Interop:** round-trips with GnuPG; passes the OpenPGP interoperability suite.
- **Privacy:** zero telemetry, local-first, no key escrow.
- **Supply chain:** reproducible builds, signed/notarized installers, SBOM,
  pinned/audited deps, published signing key.
- **Accessibility & i18n:** keyboard nav, screen-reader labels, translatable strings.
- **Packaging:** notarized `.dmg`/`.pkg` (Apple Developer ID), signed `.msi`/`.exe`
  (Authenticode/EV) — paid certs needed for no-warning installs.

## Threat model (summary)
- **Protect:** data at rest, data in transit (mail/files), private keys.
- **Adversaries:** device thieves, network eavesdroppers, malicious mail servers.
- **Out of scope (v1):** compromised OS/kernel, on-device supply chain,
  messaging metadata-resistance (revisit in F/G).

## Roadmap (dependency-ordered)
- **Phase 0 — Foundation:** `mc-core` (Sequoia) + Tauri shell on Win/Mac; key
  create/import/export; encrypt/decrypt/sign text. Proves the stack end-to-end.
- **Phase 1 (v1.0) — Core suite [P1]:** key management, file/folder ops,
  clipboard/text, hardware tokens, Finder/Explorer integration, **menu-bar /
  system-tray presence**, signed installers.
- **Phase 2 — Email + Vault [P2].**
- **Phase 3 — Mobile [P3]:** iOS + Android via Tauri.
- **Phase 4 — Messaging [P3/P4]:** MLS/Signal + backend.
- **Phase 5 — Voice/Video [P4]:** WebRTC + TURN/SFU.

## Risks to flag early
- Mail-client plug-ins vary wildly per platform/OS version.
- Mobile background execution + push; app-store crypto/export paperwork.
- Code-signing certs (Apple $99/yr; Windows EV ~$200–400/yr) for no-warning installs.
- A real **third-party security audit** before promoting for serious use.

## Success criteria
- A non-technical user can generate a key, encrypt a file for someone, and verify
  a signature **without a manual**.
- Output decrypts cleanly in **GnuPG** and vice-versa.
- Signed/notarized installers run on stock Windows + macOS with **no warnings**;
  releases are reproducible and signature-verifiable.
