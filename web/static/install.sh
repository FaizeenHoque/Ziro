#!/bin/bash
set -e

REPO="FaizeenHoque/ziro"
BINARY="ziro"

echo "Installing $BINARY..."

# --- detect platform -------------------------------------------------------
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux)
        case "$ARCH" in
            x86_64) ASSET="ziro-linux-x86_64" ;;
            *) echo "Error: unsupported Linux architecture: $ARCH"; exit 1 ;;
        esac
        ;;
    Darwin)
        case "$ARCH" in
            x86_64) ASSET="ziro-macos-x86_64" ;;
            arm64)  ASSET="ziro-macos-aarch64" ;;
            *) echo "Error: unsupported macOS architecture: $ARCH"; exit 1 ;;
        esac
        ;;
    *)
        echo "Error: unsupported OS: $OS (Windows users: download the .exe from GitHub releases directly)"
        exit 1
        ;;
esac

# --- fetch latest release tag ----------------------------------------------
if command -v jq >/dev/null 2>&1; then
    LATEST=$(curl -sf "https://api.github.com/repos/$REPO/releases/latest" | jq -r '.tag_name')
else
    echo "Warning: jq not found, falling back to grep (less reliable)"
    LATEST=$(curl -sf "https://api.github.com/repos/$REPO/releases/latest" \
        | grep '"tag_name"' \
        | head -n1 \
        | cut -d'"' -f4)
fi

if [ -z "$LATEST" ] || [ "$LATEST" = "null" ]; then
    echo "Error: Could not fetch latest release."
    exit 1
fi

URL="https://github.com/$REPO/releases/download/$LATEST/$ASSET"

echo "Downloading $BINARY $LATEST ($ASSET)..."
echo "URL: $URL"

TMP_FILE="$(mktemp)"
cleanup() { rm -f "$TMP_FILE"; }
trap cleanup EXIT

if ! curl -Lf "$URL" -o "$TMP_FILE"; then
    echo "Error: Binary not found at $URL"
    echo "Make sure the release includes a '$ASSET' asset."
    exit 1
fi

if [ ! -s "$TMP_FILE" ]; then
    echo "Error: Downloaded file is empty."
    exit 1
fi

chmod +x "$TMP_FILE"

# --- install: prefer /usr/local/bin, fall back to ~/.local/bin -------------
if [ -w "/usr/local/bin" ] || sudo -n true 2>/dev/null; then
    if sudo mv "$TMP_FILE" "/usr/local/bin/$BINARY" 2>/dev/null || mv "$TMP_FILE" "/usr/local/bin/$BINARY" 2>/dev/null; then
        trap - EXIT
        echo "Installed $BINARY $LATEST to /usr/local/bin/$BINARY"
        exit 0
    fi
fi

echo "No write access to /usr/local/bin, installing to ~/.local/bin instead..."
mkdir -p "$HOME/.local/bin"
mv "$TMP_FILE" "$HOME/.local/bin/$BINARY"
trap - EXIT
echo "Installed $BINARY $LATEST to $HOME/.local/bin/$BINARY"

case ":$PATH:" in
    *":$HOME/.local/bin:"*) ;;
    *) echo "Note: $HOME/.local/bin is not on your PATH. Add this to your shell rc:"
       echo "  export PATH=\"\$HOME/.local/bin:\$PATH\"" ;;
esac