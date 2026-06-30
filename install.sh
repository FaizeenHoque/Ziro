#!/bin/bash
set -e

REPO="FaizeenHoque/ziro"
BINARY="ziro"

echo "Installing $BINARY..."

# Get latest release tag from GitHub API
LATEST=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" \
    | grep '"tag_name"' \
    | cut -d'"' -f4)

if [ -z "$LATEST" ]; then
    echo "Error: Could not fetch latest release."
    exit 1
fi

URL="https://github.com/$REPO/releases/download/$LATEST/$BINARY"

curl -L "$URL" -o "/tmp/$BINARY"
chmod +x "/tmp/$BINARY"
sudo mv "/tmp/$BINARY" "/usr/local/bin/$BINARY"

echo "Installed $BINARY $LATEST to /usr/local/bin/$BINARY"