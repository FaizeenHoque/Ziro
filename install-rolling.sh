#!/bin/bash
set -e

REPO="FaizeenHoque/ziro"
BINARY="ziro"

echo "Installing $BINARY (rolling)..."

# Check dependencies
if ! command -v cargo &> /dev/null; then
    echo "Error: Rust/Cargo not installed. Install from https://rustup.rs"
    exit 1
fi

if ! command -v git &> /dev/null; then
    echo "Error: git not installed."
    exit 1
fi

TMP_DIR=$(mktemp -d)
trap "rm -rf $TMP_DIR" EXIT

echo "Cloning latest main..."
git clone --depth=1 "https://github.com/$REPO.git" "$TMP_DIR"

echo "Building..."
cd "$TMP_DIR"
cargo build --release

sudo mv "./target/release/$BINARY" "/usr/local/bin/$BINARY"

echo "Installed $BINARY (rolling) from latest main"