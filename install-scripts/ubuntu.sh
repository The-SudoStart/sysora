#!/bin/bash
set -euo pipefail

REPO="The-SudoStart/sysora"
VERSION="${1:-latest}"

echo "📥 Checking for latest Sysora release..."

if [[ "$VERSION" =~ ^https?:// ]]; then
  DL_URL="$VERSION"
  TAG=$(echo "$DL_URL" | grep -o 'download/[^/]*' | cut -d/ -f2)
elif [[ "$VERSION" = "latest" ]]; then
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

ARCH=$(uname -m)
if [[ "$ARCH" = "x86_64" ]]; then
  PKG_ARCH="amd64"
elif [[ "$ARCH" = "aarch64" ]] || [[ "$ARCH" = "arm64" ]]; then
  PKG_ARCH="arm64"
else
  echo "❌ Unsupported architecture: $ARCH"
  exit 1
fi

if [[ -z "${DL_URL:-}" ]]; then
  DL_URL="https://github.com/$REPO/releases/download/${TAG}/Sysora_${PKG_VER}_${PKG_ARCH}.deb"

  echo "🔍 Checking release asset: $DL_URL"
  if ! curl -sI -f "$DL_URL" > /dev/null; then
    # Fallback: check GitHub API assets for matching architecture deb file
    ALT_URL=$(curl -s "https://api.github.com/repos/$REPO/releases/tags/$TAG" | grep -o "\"browser_download_url\": *\"[^\"]*${PKG_ARCH}\\.deb\"" | head -1 | cut -d '"' -f 4 || true)
    if [[ -n "$ALT_URL" ]]; then
      DL_URL="$ALT_URL"
    else
      echo "❌ Could not find .deb package for $PKG_ARCH in release $TAG ($DL_URL)"
      echo "   Visit https://github.com/$REPO/releases to verify available assets"
      exit 1
    fi
  fi
fi

TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

echo "⬇️ Downloading Sysora ($TAG)..."
curl -fsSL "$DL_URL" -o "$TMP/sysora.deb"

if [[ ! -s "$TMP/sysora.deb" ]]; then
  echo "❌ Downloaded package is empty or missing."
  exit 1
fi

echo "🔍 Verifying package integrity..."
if ! curl -fsSL "${DL_URL}.sha256" -o "$TMP/sysora.deb.sha256" 2>/dev/null; then
  if [[ "${SYSORA_INSECURE_NO_CHECKSUM:-0}" = "1" ]]; then
    echo "⚠️ Warning: Release checksum not found; proceeding because SYSORA_INSECURE_NO_CHECKSUM=1."
  else
    echo "❌ Required release checksum (${DL_URL}.sha256) could not be retrieved."
    echo "   Failing closed to prevent installation of unverified package."
    exit 1
  fi
else
  echo "🔐 Verifying SHA256 checksum..."
  EXPECTED_SHA=$(awk '{print $1}' "$TMP/sysora.deb.sha256")
  ACTUAL_SHA=$(sha256sum "$TMP/sysora.deb" | awk '{print $1}')
  if [[ -z "$EXPECTED_SHA" ]] || [[ "$EXPECTED_SHA" != "$ACTUAL_SHA" ]]; then
    echo "❌ Checksum verification failed!"
    echo "   Expected: $EXPECTED_SHA"
    echo "   Actual:   $ACTUAL_SHA"
    exit 1
  fi
  echo "✅ Checksum verified."
fi

# Validate Debian package structure before root execution
if ! dpkg-deb -I "$TMP/sysora.deb" >/dev/null 2>&1; then
  echo "❌ Downloaded file is not a valid Debian archive."
  exit 1
fi

echo "📦 Installing Sysora package..."
if command -v pkexec >/dev/null 2>&1; then
  SUDO="pkexec"
else
  SUDO="sudo"
  echo "⚠️ Using sudo. If this is running in background, pkexec is recommended."
fi

if ! $SUDO dpkg -i "$TMP/sysora.deb"; then
  echo "🔧 Fixing dependencies via apt-get..."
  $SUDO apt-get install -f -y
fi

# Update local version file in app data dir if exists
APP_DATA_DIR="$HOME/.local/share/sysora"
if [[ -d "$APP_DATA_DIR" ]]; then
  echo "$VER" > "$APP_DATA_DIR/version.txt" || true
fi

echo "✅ Sysora updated successfully to $TAG!"
