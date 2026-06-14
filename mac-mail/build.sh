#!/bin/zsh
# Build the Rust FFI lib, stage it, and generate the Xcode project.
# Requires full Xcode active:  sudo xcode-select -s /Applications/Xcode.app
set -e
cd "${0:A:h}"
export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"

echo "==> Building mc-ffi (release)…"
( cd ../mc-ffi && cargo build --release )

mkdir -p vendor
cp ../mc-ffi/target/release/libmc_ffi.a vendor/libmc_ffi.a
cp ../mc-ffi/include/mc.h Sources/Shared/mc.h

echo "==> Generating Xcode project…"
xcodegen generate

echo ""
echo "Generated MonadnockMail.xcodeproj"
echo "Open it in Xcode and press Run, or build from the command line:"
echo "  xcodebuild -project MonadnockMail.xcodeproj -scheme MonadnockMail -configuration Debug build"
