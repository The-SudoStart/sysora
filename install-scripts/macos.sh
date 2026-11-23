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

echo "📦 Mounting installer..."
hdiutil attach "$TMP/Sysora.dmg" -mountpoint "$TMP/mount" -quiet
APP_PATH=$(find "$TMP/mount" -maxdepth 1 -name "*.app" -print -quit)
if [[ -z "$APP_PATH" ]]; then
  echo "❌ Could not find Sysora.app in mounted DMG"
  hdiutil detach "$TMP/mount" -quiet
  exit 1
fi

TARGET="/Applications/Sysora.app"
echo "📦 Copying to /Applications..."
rm -rf "$TARGET"
cp -R "$APP_PATH" /Applications/
xattr -dr com.apple.quarantine "$TARGET" 2>/dev/null || true
hdiutil detach "$TMP/mount" -quiet

APP_DATA_DIR="$HOME/Library/Application Support/sysora"
if [[ -d "$APP_DATA_DIR" ]]; then
  echo "$VER" > "$APP_DATA_DIR/version.txt" || true
fi

echo "✅ Sysora updated successfully to $TAG!"
