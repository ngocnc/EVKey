#!/usr/bin/env bash
# Builds the universal (arm64 + x86_64) EVKey.app and dmg: Rust core -> universal staticlib, SwiftPM app,
# bundle assembly, ad-hoc codesign, dmg. Output lands in build/ at the repo root.
# Requires macOS with Command Line Tools (Swift 6+), rustup and cargo. Run from anywhere.
set -euo pipefail

# Command Line Tools alone ship the newest SDK without the SwiftUIMacros plugin (@State fails to build), so pin the
# newest 26.x SDK there. Full Xcode installs are unaffected.
if [ "$(xcode-select -p)" = "/Library/Developer/CommandLineTools" ] && [ -z "${SDKROOT:-}" ]; then
  sdk="$(ls -d /Library/Developer/CommandLineTools/SDKs/MacOSX26*.sdk 2>/dev/null | sort -V | tail -n 1 || true)"
  [ -n "$sdk" ] && export SDKROOT="$sdk"
fi

root="$(cd "$(dirname "$0")/.." && pwd)"
build="$root/build"
version="$(sed -n '/^\[workspace\.package\]/,/^\[/{s/^version = "\(.*\)"/\1/p;}' "$root/Cargo.toml" | head -n 1)"
if [ -z "$version" ]; then
  echo "version not found in [workspace.package] of Cargo.toml" >&2
  exit 1
fi
echo "EVKey version $version"

# 1. Rust core: one staticlib per arch, then a universal libevkey_core.a where Package.swift links from.
rustup target add aarch64-apple-darwin x86_64-apple-darwin
for triple in aarch64-apple-darwin x86_64-apple-darwin; do
  cargo build --manifest-path "$root/Cargo.toml" -p evkey-core --release --target "$triple"
done
mkdir -p "$root/target/release"
lipo -create \
  "$root/target/aarch64-apple-darwin/release/libevkey_core.a" \
  "$root/target/x86_64-apple-darwin/release/libevkey_core.a" \
  -output "$root/target/release/libevkey_core.a"

# 2. SwiftPM universal executable.
(cd "$root/macos" && swift build -c release --arch arm64 --arch x86_64 --product EVKey)
bin_dir="$(cd "$root/macos" && swift build -c release --arch arm64 --arch x86_64 --show-bin-path)"

# 3. Assemble the .app bundle.
# Assembled and signed in a temp dir: a Desktop/iCloud folder gets FinderInfo/fileprovider xattrs added behind our back,
# which makes codesign fail with "detritus not allowed" and leaves the app unsigned.
rm -rf "$build"; mkdir -p "$build"
work="$(mktemp -d)"
app="$work/EVKey.app"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
cp "$bin_dir/EVKey" "$app/Contents/MacOS/EVKey"
cp "$root/macos/Info.plist" "$app/Contents/Info.plist"
plutil -replace CFBundleShortVersionString -string "$version" "$app/Contents/Info.plist"
plutil -replace CFBundleVersion -string "$version" "$app/Contents/Info.plist"
cp "$root/crates/app/icons/icon.icns" "$app/Contents/Resources/icon.icns"
# SwiftPM resource bundles (e.g. EVKey_EVKeyKit.bundle) must sit next to the executable's resources.
for res in "$bin_dir"/*.bundle; do
  [ -e "$res" ] && cp -R "$res" "$app/Contents/Resources/"
done
plutil -lint "$app/Contents/Info.plist" >/dev/null

# 4. Signature. A stable identity (a local self-signed certificate named "EVKey Local Signing", see README) keeps the
# macOS Accessibility grant across rebuilds, because the designated requirement is then the certificate, not the code
# hash. Without it, fall back to ad-hoc ("-"), which loses the grant on every rebuild.
xattr -cr "$app"
identity="-"
if security find-identity -v -p codesigning | grep -q "EVKey Local Signing"; then identity="EVKey Local Signing"; fi
codesign --force --deep -s "$identity" -i com.chrisnguyen.evkey "$app"
codesign --verify --deep --strict "$app"

# 5. dmg with the app and an /Applications shortcut.
stage="$(mktemp -d)"
trap 'rm -rf "$stage" "$work"' EXIT
cp -R "$app" "$stage/"
ln -s /Applications "$stage/Applications"
dmg="$build/EVKey-${version}-universal.dmg"
hdiutil create -quiet -volname "EVKey $version" -srcfolder "$stage" -ov -format UDZO "$dmg"

shasum -a 256 "$dmg"
echo "dmg: $dmg"
ditto --noextattr "$app" "$build/EVKey.app"
