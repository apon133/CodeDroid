#!/bin/bash
set -e

# Color definitions
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo "CodeDroid: Starting Rust API compilation and deployment..."

# Ensure cargo-zigbuild is installed
if ! command -v cargo-zigbuild &> /dev/null; then
    echo -e "${RED}Error: cargo-zigbuild is not installed.${NC}"
    echo "Please install it by running: cargo install cargo-zigbuild"
    exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
API_DIR="$SCRIPT_DIR/codedroid_api"
ASSETS_DIR="$SCRIPT_DIR/apps/flutter_android/assets/linux"

# Build target lists
TARGETS=("aarch64-unknown-linux-musl" "x86_64-unknown-linux-musl")
ARCHS=("aarch64" "x86_64")

cd "$API_DIR"

for i in "${!TARGETS[@]}"; do
    TARGET="${TARGETS[$i]}"
    ARCH="${ARCHS[$i]}"
    
    echo "--------------------------------------------------"
    echo -e "Building for target: ${GREEN}${TARGET}${NC} (${ARCH})..."
    echo "--------------------------------------------------"
    
    # Run the zigbuild command
    cargo zigbuild --target "$TARGET" --release
    
    # Destination directory path
    DEST_DIR="$ASSETS_DIR/$ARCH"
    mkdir -p "$DEST_DIR"
    
    # Copy the compiled binary
    SRC_BIN="target/$TARGET/release/codedroid_api"
    DEST_BIN="$DEST_DIR/codedroid_api"
    
    if [ -f "$SRC_BIN" ]; then
        cp "$SRC_BIN" "$DEST_BIN"
        echo -e "${GREEN}Copied successfully:${NC} $DEST_BIN"
    else
        echo -e "${RED}Error: Binary not found at ${SRC_BIN}${NC}"
        exit 1
    fi
done

echo "--------------------------------------------------"
echo -e "${GREEN}All targets built and deployed successfully!${NC}"
echo "--------------------------------------------------"
