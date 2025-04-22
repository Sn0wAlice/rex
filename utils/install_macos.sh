#!/bin/bash

set -e

REPO="Sn0wAlice/rex"
INSTALL_DIR="/usr/local/bin"
TMP_DIR=$(mktemp -d)
ARCH=$(uname -m)

# Map architecture
if [[ "$ARCH" == "arm64" ]]; then
    PLATFORM="macOS-arm64"
elif [[ "$ARCH" == "x86_64" ]]; then
    PLATFORM="macOS-x86_64"
else
    echo "Unsupported architecture: $ARCH"
    exit 1
fi

# Get the latest release tag
echo "Fetching latest release from GitHub..."
LATEST=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep tag_name | cut -d '"' -f 4)

# Build filenames
FILE="rex-${PLATFORM}.tar.gz"
CHECKSUM="${FILE}.sha256"
URL="https://github.com/$REPO/releases/download/${LATEST}/${FILE}"
CHECKSUM_URL="https://github.com/$REPO/releases/download/${LATEST}/${CHECKSUM}"

echo "Downloading $FILE..."
curl -L "$URL" -o "$TMP_DIR/$FILE"
curl -L "$CHECKSUM_URL" -o "$TMP_DIR/$CHECKSUM"

echo "Verifying checksum..."
cd "$TMP_DIR"
sha256sum -c "$CHECKSUM"

echo "Extracting and installing..."
tar -xzf "$FILE"
chmod +x rex
sudo mv rex "$INSTALL_DIR/rex"

echo "Installed rex to $INSTALL_DIR/rex"
echo "Thank you for using rex!"

rex version