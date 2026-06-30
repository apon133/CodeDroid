#!/bin/bash
# =============================================================================
# CodeDroid Termux installer script
# =============================================================================
# Install command:
#   curl -sL https://raw.githubusercontent.com/apon133/CodeDroid/main/install.sh | bash
# =============================================================================

set -e

# Terminal Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}==================================================${NC}"
echo -e "${GREEN}🚀       CodeDroid Termux installer script       🚀${NC}"
echo -e "${BLUE}==================================================${NC}"

# 1. Architecture Detection
ARCH=$(uname -m)
case "$ARCH" in
    aarch64|arm64)
        BINARY_ARCH="aarch64"
        ;;
    x86_64|amd64)
        BINARY_ARCH="x86_64"
        ;;
    *)
        echo -e "${RED}❌ Unsupported architecture: $ARCH${NC}"
        echo "Currently, only aarch64 (ARM64) and x86_64 are supported."
        exit 1
        ;;
esac

# 2. Dependency Check (curl)
if ! command -v curl &> /dev/null; then
    echo -e "${YELLOW}🔍 curl not found. Installing curl...${NC}"
    pkg update && pkg install -y curl || {
        echo -e "${RED}❌ Failed to install curl. Please run 'pkg install curl' manually.${NC}"
        exit 1;
    }
fi

# 3. Create install directories
INSTALL_DIR="$HOME/.codedroid"
mkdir -p "$INSTALL_DIR"

# 4. Download pre-compiled binary
BINARY_URL="https://raw.githubusercontent.com/apon133/CodeDroid/main/apps/flutter_android/assets/linux/${BINARY_ARCH}/codedroid_api"

echo -e "${BLUE}⬇️ Downloading CodeDroid API Binary (${BINARY_ARCH})...${NC}"
curl -L --progress-bar -o "$INSTALL_DIR/codedroid-api" "$BINARY_URL"

# Make it executable
chmod +x "$INSTALL_DIR/codedroid-api"

# 5. Create CLI Wrapper Script in $PREFIX/bin
TERMUX_BIN_DIR="/data/data/com.termux/files/usr/bin"
if [ -d "$TERMUX_BIN_DIR" ]; then
    WRAPPER_PATH="$TERMUX_BIN_DIR/codedroid"
else
    # Fallback to local bin if not in Termux
    mkdir -p "$HOME/.local/bin"
    WRAPPER_PATH="$HOME/.local/bin/codedroid"
    
    # Check if ~/.local/bin is in PATH
    if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
        echo -e "${YELLOW}⚠️  Note: ~/.local/bin is not in your PATH. You may need to add it.${NC}"
    fi
fi

echo -e "${BLUE}⚙️ Installing 'codedroid' command wrapper...${NC}"

cat > "$WRAPPER_PATH" << 'EOF'
#!/bin/bash

# Configuration
INSTALL_DIR="$HOME/.codedroid"
BINARY_PATH="$INSTALL_DIR/codedroid-api"
INSTALLER_URL="https://raw.githubusercontent.com/apon133/CodeDroid/main/install.sh"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
NC='\033[0m'

show_help() {
    echo -e "${GREEN}CodeDroid Termux CLI Wrapper${NC}"
    echo ""
    echo "Usage:"
    echo "  codedroid [command]"
    echo ""
    echo "Commands:"
    echo "  start       Start the CodeDroid API server (default)"
    echo "  update      Update CodeDroid API to the latest version"
    echo "  uninstall   Uninstall CodeDroid API and cleanup files"
    echo "  help        Show this help message"
    echo ""
}

start_server() {
    if [ ! -f "$BINARY_PATH" ]; then
        echo -e "${RED}❌ Binary not found at $BINARY_PATH.${NC}"
        echo "Please re-run the installation script:"
        echo "curl -sL $INSTALLER_URL | bash"
        exit 1
    fi

    echo -e "${BLUE}==================================================${NC}"
    echo -e "${GREEN}      🚀 Starting CodeDroid API Server 🚀         ${NC}"
    echo -e "${BLUE}==================================================${NC}"
    echo "📡 Server starting on: http://0.0.0.0:3000"
    echo "💻 Web IDE: Visit https://codedroid.netlify.app and connect!"
    echo -e "${BLUE}--------------------------------------------------${NC}"
    
    # Run the binary
    exec "$BINARY_PATH" "$@"
}

update_binary() {
    echo -e "${YELLOW}🔄 Checking for updates and reinstalling...${NC}"
    curl -sL "$INSTALLER_URL" | bash
}

uninstall_codedroid() {
    read -p "Are you sure you want to uninstall CodeDroid? (y/N): " confirm
    if [[ "$confirm" =~ ^[Yy]$ ]]; then
        echo -e "${RED}🗑️ Uninstalling CodeDroid...${NC}"
        rm -rf "$INSTALL_DIR"
        
        # Remove wrapper
        # Detect wrapper path
        MY_PATH="$(which codedroid 2>/dev/null || echo '')"
        if [ -n "$MY_PATH" ] && [ -f "$MY_PATH" ]; then
            rm -f "$MY_PATH"
        fi
        
        echo -e "${GREEN}✅ CodeDroid successfully uninstalled!${NC}"
    else
        echo "Uninstall cancelled."
    fi
}

# Main Command Router
CMD="${1:-start}"

case "$CMD" in
    start)
        # Shift first argument to pass remaining args to binary
        shift || true
        start_server "$@"
        ;;
    update)
        update_binary
        ;;
    uninstall)
        uninstall_codedroid
        ;;
    help|--help|-h)
        show_help
        ;;
    *)
        # If it's a flag (e.g. --some-flag) and we didn't match anything,
        # pass it directly to start server
        if [[ "$CMD" == -* ]]; then
            start_server "$@"
        else
            echo -e "${RED}❌ Unknown command: $CMD${NC}"
            show_help
            exit 1
        fi
        ;;
esac
EOF

chmod +x "$WRAPPER_PATH"

echo -e "${GREEN}✅ CodeDroid installed successfully!${NC}"
echo ""
echo -e "You can now run CodeDroid using this command:"
echo -e "  ${GREEN}codedroid${NC}"
echo ""
echo -e "To update CodeDroid to the latest version in the future, just run:"
echo -e "  ${GREEN}codedroid update${NC}"
echo ""
echo -e "To start, simply type:"
echo -e "  ${GREEN}codedroid${NC}"
echo -e "${BLUE}==================================================${NC}"
