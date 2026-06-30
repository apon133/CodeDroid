#!/bin/bash
set -e

# Color definitions
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Directories
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
API_DIR="$SCRIPT_DIR/codedroid_api"
FLUTTER_ASSETS_DIR="$SCRIPT_DIR/apps/flutter_android/assets/linux"

echo -e "${BLUE}==================================================${NC}"
echo -e "${GREEN}🚀 CodeDroid Android/Linux Cross-Compilation Tool${NC}"
echo -e "${BLUE}==================================================${NC}"

# Ensure we are compiling in the correct subdirectory so Cargo picks up .cargo/config.toml
cd "$API_DIR"

# 1. Compile aarch64
echo -e "\n${BLUE}[1/4] Compiling for aarch64 (ARM 64-bit)...${NC}"
cargo build --release --target aarch64-unknown-linux-musl

# 2. Compile x86_64
echo -e "\n${BLUE}[2/4] Compiling for x86_64 (Intel/AMD 64-bit)...${NC}"
cargo build --release --target x86_64-unknown-linux-musl

# 3. Strip binaries to minimize size
echo -e "\n${BLUE}[3/4] Stripping binaries to reduce size...${NC}"
AARCH64_BIN="$API_DIR/target/aarch64-unknown-linux-musl/release/codedroid_api"
X86_64_BIN="$API_DIR/target/x86_64-unknown-linux-musl/release/codedroid_api"

if command -v aarch64-linux-musl-strip &> /dev/null; then
    echo "Stripping aarch64 binary..."
    aarch64-linux-musl-strip "$AARCH64_BIN"
else
    echo "⚠️ Warning: aarch64-linux-musl-strip not found, skipping strip."
fi

if command -v x86_64-linux-musl-strip &> /dev/null; then
    echo "Stripping x86_64 binary..."
    x86_64-linux-musl-strip "$X86_64_BIN"
else
    echo "⚠️ Warning: x86_64-linux-musl-strip not found, skipping strip."
fi

# 4. Copy to Flutter Assets
echo -e "\n${BLUE}[4/4] Deploying binaries to Flutter assets...${NC}"
mkdir -p "$FLUTTER_ASSETS_DIR/aarch64"
mkdir -p "$FLUTTER_ASSETS_DIR/x86_64"

cp "$AARCH64_BIN" "$FLUTTER_ASSETS_DIR/aarch64/codedroid_api"
cp "$X86_64_BIN" "$FLUTTER_ASSETS_DIR/x86_64/codedroid_api"

echo -e "\n${GREEN}✅ Success! Android/Linux binaries compiled, stripped, and deployed to Flutter assets.${NC}"
echo -e "${BLUE}==================================================${NC}"
