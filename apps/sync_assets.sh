#!/bin/bash
set -e

# Get the script's directory and root workspace
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
WORKSPACE_DIR="$SCRIPT_DIR/.."
FRONTEND_DIR="$WORKSPACE_DIR/codedroid_frontend"
FLUTTER_WWW_DIR="$SCRIPT_DIR/flutter_android/assets/www"
TAURI_SRC_DIR="$SCRIPT_DIR/tauri_desktop/src"

echo "============================================="
echo "🔄 CodeDroid Asset Sync & Build Tool"
echo "============================================="

# Ensure trunk is available
if ! command -v trunk &> /dev/null; then
    echo "❌ Error: 'trunk' CLI is not installed. Please install it first."
    exit 1
fi

# Step 1: Build the frontend with Trunk
echo "📦 Step 1: Building codedroid_frontend with Trunk (Release mode)..."
cd "$FRONTEND_DIR"
trunk build --release

# Check if dist folder was created successfully
if [ ! -d "$FRONTEND_DIR/dist" ]; then
    echo "❌ Error: Build finished but 'dist' directory not found!"
    exit 1
fi

# Step 2: Clean target directories
echo "🧹 Step 2: Cleaning target directories..."
rm -rf "$FLUTTER_WWW_DIR"
rm -rf "$TAURI_SRC_DIR"

mkdir -p "$FLUTTER_WWW_DIR"
mkdir -p "$TAURI_SRC_DIR"

# Step 3: Copy assets to Flutter (Android)
echo "🚀 Step 3: Syncing to Flutter Android Assets..."
cp -R "$FRONTEND_DIR/dist/"* "$FLUTTER_WWW_DIR/"
echo "✅ Flutter assets updated."

# Step 4: Copy assets to Tauri (Desktop)
echo "🚀 Step 4: Syncing to Tauri Desktop Source..."
cp -R "$FRONTEND_DIR/dist/"* "$TAURI_SRC_DIR/"
echo "✅ Tauri src updated."

echo "============================================="
echo "✨ Success! Latest assets synced to all apps."
echo "============================================="
echo "To run your Flutter app (Android):"
echo "  cd $SCRIPT_DIR/flutter_android && flutter run"
echo ""
echo "To run your Tauri app (Desktop):"
echo "  cd $SCRIPT_DIR/tauri_desktop && pnpm tauri dev"
echo "============================================="
