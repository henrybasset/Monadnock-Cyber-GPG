# Mac setup — `mc` and right-click "Decrypt in Mail"

## 1. Install the command-line core

Double-click **`install-mac.command`** (or run it). It builds `mc` and puts it on
your PATH. `mc` shares the desktop app's keyring, so keys you create in the app
just work.

```sh
mc list                  # your keys
echo hi | mc encrypt --to <FINGERPRINT> | mc decrypt
pbpaste | mc decrypt     # decrypt whatever you've copied
```

## 2. Right-click "Decrypt with Monadnock GPG" (works inside Mail)

This adds a system-wide right-click action, so in **Mail** you can select an
encrypted block → right-click → **Decrypt with Monadnock GPG** → the plaintext
pops up. ~1 minute to set up:

1. Open **Automator** → **New** → **Quick Action**.
2. Set **"Workflow receives current"** to **text** in **any application**.
3. From the left list, drag **"Run Shell Script"** into the workflow.
4. Set **Shell** to `/bin/zsh` and **Pass input** to **to stdin**.
5. Paste this script:

   ```sh
   PT="$(/opt/homebrew/bin/mc decrypt 2>/dev/null)"
   if [ -z "$PT" ]; then
     osascript -e 'display alert "Monadnock Cyber GPG" message "Could not decrypt — no matching key, or the selection is not an encrypted message."'
     exit 0
   fi
   F="$(mktemp /tmp/mcgpg.XXXXXX)"
   printf '%s' "$PT" > "$F"
   osascript -e "set t to (do shell script \"cat \" & quoted form of \"$F\")" -e 'display alert "Decrypted ✓" message t'
   rm -f "$F"
   ```

   (If `install-mac.command` reported a different path than `/opt/homebrew/bin/mc`,
   use that path instead.)

6. **Save** as **"Decrypt with Monadnock GPG"**.

Now: in Mail, select the encrypted text → **right-click → Quick Actions →
Decrypt with Monadnock GPG**. First run, macOS asks permission — click OK.

> This is the no-Xcode step-saver. The fully inline experience (auto-decrypt
> banner inside Mail) comes from the **MailKit extension** — see below.

## 3. The real in-Mail extension (MailKit) — needs Xcode

For encrypt/decrypt **inline inside Mail** (a security banner, automatic), the
modern supported path is a **MailKit app extension**. It requires **full Xcode**
(not just Command Line Tools) and your Apple Developer ID to sign:

```sh
# after installing Xcode from the App Store:
sudo xcode-select -s /Applications/Xcode.app
```

Then we build the extension (Swift) that calls this same `mc-core` crypto.
