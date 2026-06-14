# Monadnock Cyber GPG — browser extension

OpenPGP in your browser and webmail (Gmail, Outlook-web, …), built on
[OpenPGP.js](https://openpgpjs.org). Same standard as the desktop app, so keys
move between them by export/import.

## What it does

- **Popup** (toolbar icon): generate / import / share / delete keys, and
  encrypt or decrypt text.
- **Right-click → "Decrypt with Monadnock GPG"**: select an encrypted block in
  any webmail and decrypt it in place — the plaintext appears in a small panel.

> v1 keeps its own keyring (in the browser). Import your key from the desktop
> app (Keys → Share copies the public key; export your secret separately) to use
> the same identity. A future version can share the desktop keyring directly.

## Install (developer / unpacked)

1. Open **Chrome** (or Edge/Brave) → `chrome://extensions`.
2. Turn on **Developer mode** (top-right).
3. Click **Load unpacked** and choose this `browser-extension/` folder.
4. The padlock icon appears in the toolbar. Click it to manage keys and
   encrypt/decrypt; right-click selected ciphertext in webmail to decrypt.

(Firefox: `about:debugging` → This Firefox → Load Temporary Add-on → pick
`manifest.json`.)

## Notes

- Keys are stored in the browser's extension storage, on this machine only.
- v1 assumes keys have no passphrase (matching the desktop app's keys).
