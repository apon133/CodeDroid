#!/bin/bash

# Exit immediately if a command exits with a non-zero status
set -e

# Detect OS
OS_NAME="$(uname -s)"
BINARY_NAME="codedroid_api"
TARGET_BINARY="codedroid-api"

if [[ "$OS_NAME" == *"NT"* || "$OS_NAME" == *"MINGW"* || "$OS_NAME" == *"MSYS"* || "$OS_NAME" == *"CYGWIN"* ]]; then
    BINARY_NAME="codedroid_api.exe"
    TARGET_BINARY="codedroid-api.exe"
fi

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BINARY_PATH="$SCRIPT_DIR/$TARGET_BINARY"

# Function to build the binary
build_binary() {
    echo "=================================================="
    echo "      🛠️ Building CodeDroid API Binary 🛠️        "
    echo "=================================================="
    
    if ! command -v cargo &> /dev/null; then
        echo "❌ Error: 'cargo' (Rust package manager) is not installed!"
        echo "   Please install Rust to build from source: https://rustup.rs/"
        exit 1
    fi

    echo "📦 Running cargo build in release mode..."
    cargo build --release --manifest-path "$SCRIPT_DIR/codedroid_api/Cargo.toml"

    COMPILED_PATH="$SCRIPT_DIR/codedroid_api/target/release/$BINARY_NAME"

    if [ -f "$COMPILED_PATH" ]; then
        if [ -f "$BINARY_PATH" ]; then
            echo "🗑️ Removing old binary from root..."
            rm -f "$BINARY_PATH"
        fi
        echo "💾 Copying new binary to root directory..."
        cp "$COMPILED_PATH" "$BINARY_PATH"
        
        if [ "$TARGET_BINARY" == "codedroid-api" ]; then
            chmod +x "$BINARY_PATH"
        fi
        echo "🎉 Build successful!"
        echo "--------------------------------------------------"
    else
        echo "❌ Error: Compiled binary was not found at $COMPILED_PATH"
        exit 1
    fi
}

# Check flags
if [[ "$1" == "--build" || "$1" == "-b" ]]; then
    build_binary
fi

echo "=================================================="
echo "      🚀 Starting CodeDroid API Server 🚀         "
echo "=================================================="

# Check if the binary exists
if [ ! -f "$BINARY_PATH" ]; then
    echo "🔍 Binary '$TARGET_BINARY' not found in the root directory."
    
    # Check if Cargo is available to compile it automatically
    if command -v cargo &> /dev/null; then
        echo "💡 Rust/Cargo detected! Attempting to compile from source..."
        build_binary
    else
        echo "❌ Error: Pre-compiled binary '$TARGET_BINARY' not found and Rust/Cargo is not installed."
        echo "   Please either:"
        echo "   1. Place a pre-compiled '$TARGET_BINARY' binary in: $SCRIPT_DIR/"
        echo "   2. Install Rust/Cargo (https://rustup.rs/) to compile it automatically."
        echo ""
        echo "💡 Note: You can download the pre-compiled binary from the GitHub releases page."
        exit 1
    fi
fi

# Make sure the binary is executable (on Linux/macOS/Termux)
if [ "$TARGET_BINARY" == "codedroid-api" ]; then
    chmod +x "$BINARY_PATH"
fi

echo "🟢 Running CodeDroid API binary directly..."
echo "📡 Server will start on: http://0.0.0.0:3000"
echo "--------------------------------------------------"

# Run the binary
"$BINARY_PATH"
