#!/bin/bash
set -e

REPO="FaizeenHoque/ziro"
BINARY="ziro"

echo "Installing $BINARY..."

LATEST=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" \
    | grep '"tag_name"' \
    | cut -d'"' -f4)

if [ -z "$LATEST" ]; then
    echo "Error: Could not fetch latest release."
    exit 1
fi

URL="https://github.com/$REPO/releases/download/$LATEST/$BINARY"

echo "Downloading $BINARY $LATEST..."
echo "URL: $URL"  

if ! curl -Lf "$URL" -o "/tmp/$BINARY"; then
    echo "Error: Binary not found at $URL"
    echo "Make sure you uploaded the binary with: gh release create"
    exit 1
fi

chmod +x "/tmp/$BINARY"
sudo mv "/tmp/$BINARY" "/usr/local/bin/$BINARY"

echo "Installed $BINARY $LATEST"