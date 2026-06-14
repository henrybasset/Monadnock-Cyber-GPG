#!/bin/zsh
# Build and install the `mc` command-line core onto your PATH.
# Double-click in Finder, or run from a terminal.
set -e
cd "${0:A:h}"
export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"

if ! command -v cargo >/dev/null 2>&1; then
  echo "Rust isn't installed. Get it from https://rustup.rs, then run this again."
  echo "Press any key to close…"; read -k1; exit 1
fi

echo "Building mc (release)… first time takes a few minutes."
( cd cli && cargo build --release )

DEST="/opt/homebrew/bin"
if [[ ! -d "$DEST" || ! -w "$DEST" ]]; then
  DEST="$HOME/.local/bin"
  mkdir -p "$DEST"
fi
cp cli/target/release/mc "$DEST/mc"

echo ""
echo "Installed: $DEST/mc"
case ":$PATH:" in
  *":$DEST:"*) ;;
  *) echo "Note: add $DEST to your PATH (e.g. add to ~/.zshrc):"
     echo "      export PATH=\"$DEST:\$PATH\"" ;;
esac
echo ""
echo "Try it:"
echo "  mc list                       # your keys (shared with the app)"
echo "  pbpaste | mc decrypt          # decrypt whatever you copied"
echo ""
echo "Right-click 'Decrypt with Monadnock GPG' in Mail: see MAC_SETUP.md."
if [[ "$1" != "--quiet" ]]; then
  echo ""; echo "Press any key to close…"; read -k1
fi
