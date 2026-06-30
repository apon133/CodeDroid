#!/bin/bash
set -e

# Detect OS
OS_NAME="$(uname -s)"
TARGET_BINARY="codedroid-api"

if [[ "$OS_NAME" == *"NT"* || "$OS_NAME" == *"MINGW"* || "$OS_NAME" == *"MSYS"* || "$OS_NAME" == *"CYGWIN"* ]]; then
    TARGET_BINARY="codedroid-api.exe"
fi

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BINARY_PATH="$SCRIPT_DIR/$TARGET_BINARY"

echo "=================================================="
echo "      🚀 Starting CodeDroid API Server 🚀         "
echo "=================================================="

# Check if the binary exists
if [ ! -f "$BINARY_PATH" ]; then
    echo "🔍 Binary '$TARGET_BINARY' not found in the root directory."
    echo "🛠️ Compiling from source..."
    
    # Run build_api.sh to compile and place it in the root
    if [ -f "$SCRIPT_DIR/build_api.sh" ]; then
        bash "$SCRIPT_DIR/build_api.sh"
    else
        echo "❌ Error: build_api.sh not found to build the binary."
        exit 1
    fi
else
    echo "🟢 Existing '$TARGET_BINARY' found. Running directly..."
fi

# Make sure the binary is executable (on Linux/macOS/Termux)
if [ "$TARGET_BINARY" == "codedroid-api" ]; then
    chmod +x "$BINARY_PATH"
fi

echo "📡 Server will start on: http://0.0.0.0:3000"
echo "--------------------------------------------------"

# Run the binary
exec "$BINARY_PATH" "$@"
