#!/bin/bash
set -euo pipefail

REPO="The-SudoStart/sysora"
VERSION="${1:-latest}"

echo "📥 Checking for latest Sysora release for macOS..."

if [[ "$VERSION" = "latest" ]]; then
  TAG=$(curl -sI "https://github.com/$REPO/releases/latest" | grep -i '^location:' | awk -F'/' '{print $NF}' | tr -d '\r\n')
  if [[ -z "$TAG" ]]; then
    TAG=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep -o '"tag_name": *"[^"]*"' | head -1 | cut -d '"' -f 4)
  fi
else
  TAG="$VERSION"
fi

if [[ -z "$TAG" ]]; then
  echo "❌ Could not determine version tag."
  exit 1
fi

VER="${TAG#v}"
TAG="v${VER}"
PKG_VER="${VER%%-rc.*}"

DL_URL="https://github.com/$REPO/releases/download/${TAG}/Sysora_${PKG_VER}_universal.dmg"

echo "🔍 Checking release asset: $DL_URL"
if ! curl -sI -f "$DL_URL" > /dev/null; then
  ALT_URL=$(curl -s "https://api.github.com/repos/$REPO/releases/tags/$TAG" | grep -o '"browser_download_url": *"[^"]*universal\.dmg"' | head -1 | cut -d '"' -f 4 || true)
  if [[ -n "$ALT_URL" ]]; then
    DL_URL="$ALT_URL"
  else
    echo "❌ Could not find macOS DMG package in release $TAG"
    exit 1
  fi
fi

TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

echo "⬇️ Downloading Sysora ($TAG)..."
curl -fsSL "$DL_URL" -o "$TMP/Sysora.dmg"

if [[ ! -s "$TMP/Sysora.dmg" ]]; then
  echo "❌ Downloaded DMG package is empty or missing."
  exit 1
fi

echo "🔍 Verifying package integrity..."
if ! curl -fsSL "${DL_URL}.sha256" -o "$TMP/Sysora.dmg.sha256" 2>/dev/null; then
  if [[ "${SYSORA_INSECURE_NO_CHECKSUM:-0}" = "1" ]]; then
    echo "⚠️ Warning: Release checksum not found; proceeding because SYSORA_INSECURE_NO_CHECKSUM=1."
  else
    echo "❌ Required release checksum (${DL_URL}.sha256) could not be retrieved."
    echo "   Failing closed to prevent installation of unverified package."
    exit 1
  fi
else
  echo "🔐 Verifying SHA256 checksum..."
  EXPECTED_SHA=$(awk '{print $1}' "$TMP/Sysora.dmg.sha256")
  ACTUAL_SHA=$(shasum -a 256 "$TMP/Sysora.dmg" | awk '{print $1}')
  if [[ -z "$EXPECTED_SHA" ]] || [[ "$EXPECTED_SHA" != "$ACTUAL_SHA" ]]; then
    echo "❌ Checksum verification failed!"
    echo "   Expected: $EXPECTED_SHA"
    echo "   Actual:   $ACTUAL_SHA"
    exit 1
  fi
  echo "✅ Checksum verified."
fi

echo "📦 Mounting installer..."
hdiutil attach "$TMP/Sysora.dmg" -mountpoint "$TMP/mount" -quiet
APP_PATH=$(find "$TMP/mount" -maxdepth 1 -name "*.app" -print -quit)
if [[ -z "$APP_PATH" ]] || [[ ! -f "$APP_PATH/Contents/Info.plist" ]]; then
  echo "❌ Could not find a valid Sysora.app bundle in mounted DMG"
  hdiutil detach "$TMP/mount" -quiet
  exit 1
fi

TARGET="/Applications/Sysora.app"
echo "📦 Copying to /Applications..."
rm -rf "$TARGET"
cp -R "$APP_PATH" /Applications/
hdiutil detach "$TMP/mount" -quiet

# Intentionally bypass macOS Gatekeeper and unverified signature checks automatically
if [[ -f "$TARGET/Contents/Info.plist" ]]; then
  echo "🔓 Applying intentional signature bypass & clearing quarantine..."
  xattr -cr "$TARGET" 2>/dev/null || true
  xattr -dr com.apple.quarantine "$TARGET" 2>/dev/null || true
  codesign --force --deep --sign - "$TARGET" 2>/dev/null || true
fi

APP_DATA_DIR="$HOME/Library/Application Support/sysora"
if [[ -d "$APP_DATA_DIR" ]]; then
  echo "$VER" > "$APP_DATA_DIR/version.txt" || true
fi

echo "✅ Sysora updated successfully to $TAG!"
