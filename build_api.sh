#!/bin/bash
set -e

# Color definitions
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo "CodeDroid: Starting Rust API compilation and deployment..."

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Detect OS
OS_NAME="$(uname -s)"
BINARY_NAME="codedroid_api"
TARGET_BINARY="codedroid-api"

if [[ "$OS_NAME" == *"NT"* || "$OS_NAME" == *"MINGW"* || "$OS_NAME" == *"MSYS"* || "$OS_NAME" == *"CYGWIN"* ]]; then
    BINARY_NAME="codedroid_api.exe"
    TARGET_BINARY="codedroid-api.exe"
fi

BINARY_PATH="$SCRIPT_DIR/$TARGET_BINARY"

# 1. Remove existing binary in the root
if [ -f "$BINARY_PATH" ]; then
    echo "🗑️ Removing existing binary: $BINARY_PATH"
    rm -f "$BINARY_PATH"
fi

# 2. Check if Cargo is installed
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Error: cargo is not installed.${NC}"
    echo "Please install Rust/Cargo: https://rustup.rs/"
    exit 1
fi

# 3. Build using cargo build --release
echo "--------------------------------------------------"
echo -e "Building ${GREEN}$TARGET_BINARY${NC} in release mode..."
echo "--------------------------------------------------"
cargo build --release --manifest-path "$SCRIPT_DIR/codedroid_api/Cargo.toml"

COMPILED_PATH="$SCRIPT_DIR/codedroid_api/target/release/$BINARY_NAME"

# 4. Move to root directory
if [ -f "$COMPILED_PATH" ]; then
    echo "💾 Moving compiled binary to root directory..."
    mv "$COMPILED_PATH" "$BINARY_PATH"
    
    if [ "$TARGET_BINARY" == "codedroid-api" ]; then
        chmod +x "$BINARY_PATH"
    fi
    echo -e "${GREEN}Build and deployment successful:${NC} $BINARY_PATH"
else
    echo -e "${RED}Error: Compiled binary not found at $COMPILED_PATH${NC}"
    exit 1
fi
