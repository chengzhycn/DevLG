#!/bin/bash

set -e

# GitHub repository information
REPO="chengzhycn/devlg"
VERSION="latest" # You can change this to a specific version if needed

# Detect OS and architecture
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

# Map architecture names
case $ARCH in
"x86_64")
    ARCH="x86_64"
    ;;
"arm64" | "aarch64")
    ARCH="aarch64"
    ;;
*)
    echo "Unsupported architecture: $ARCH"
    exit 1
    ;;
esac

# Map OS names
case $OS in
"darwin")
    OS="apple-darwin"
    ;;
"linux")
    OS="linux-gnu"
    ;;
*)
    echo "Unsupported operating system: $OS"
    exit 1
    ;;
esac

# Construct the asset name
ASSET_NAME="devlg.${ARCH}-${OS}.tar.gz"

# Create a temporary directory
TEMP_DIR=$(mktemp -d)
cd "$TEMP_DIR"

echo "Downloading $ASSET_NAME..."

# Download the asset
if ! curl -L -o "$ASSET_NAME" "https://github.com/$REPO/releases/$VERSION/download/$ASSET_NAME"; then
    echo "Failed to download $ASSET_NAME"
    rm -rf "$TEMP_DIR"
    exit 1
fi

# Extract the archive
echo "Extracting..."
tar xzf "$ASSET_NAME"

# Install the binary
echo "Installing devlg to /usr/local/bin..."
if [ ! -w "/usr/local/bin" ]; then
    echo "Need sudo permission to install to /usr/local/bin"
    sudo mv devlg /usr/local/bin/
else
    mv devlg /usr/local/bin/
fi

# Clean up
cd - >/dev/null
rm -rf "$TEMP_DIR"

echo "Installation complete! You can now use the 'devlg' command."
