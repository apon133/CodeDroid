#!/data/data/com.termux/files/usr/bin/bash
# =============================================================================
# setup_alpine.sh — Manual Alpine Linux PRoot Setup for Termux
# =============================================================================
# NOTE: The CodeDroid Flutter app sets up Alpine automatically on first launch.
# Run this script ONLY if you want to manually set up Alpine in Termux for
# the purpose of:
#   - iOS / remote device network access (Device A = your Android running API)
#   - Debugging or custom tool installation
#
# Architecture:
#   ┌──────────────────────┐      ┌─────────────────────────────────────┐
#   │  iPhone / iOS / PC   │─WiFi─▶  Android (CodeDroid Flutter App)   │
#   │  Opens in browser:   │      │  ┌─────────────────────────────┐   │
#   │  http://<ip>:8082    │      │  │ Alpine Linux (PRoot)         │   │
#   └──────────────────────┘      │  │ ├─ codedroid_api → :3000     │   │
#                                 │  │ └─ trunk serve  → :8082      │   │
#                                 │  └─────────────────────────────┘   │
#                                 └─────────────────────────────────────┘
# =============================================================================

set -e

ALPINE_VERSION="3.19"
ALPINE_ARCH="aarch64"   # arm64 — most modern Android phones
ALPINE_URL="https://dl-cdn.alpinelinux.org/alpine/v${ALPINE_VERSION}/releases/${ALPINE_ARCH}/alpine-minirootfs-${ALPINE_VERSION}.0-${ALPINE_ARCH}.tar.gz"

# Alpine lives here in Termux home (separate from app's internal storage)
ALPINE_ROOT="$HOME/alpine"
TERMUX_PREFIX="/data/data/com.termux/files/usr"

echo ""
echo "╔══════════════════════════════════════════════════════════╗"
echo "║   CodeDroid — Alpine Linux PRoot (Termux Manual Setup)  ║"
echo "╚══════════════════════════════════════════════════════════╝"
echo ""
echo "Why Alpine Linux?"
echo "  ✅ Smallest rootfs (~5 MB download)"
echo "  ✅ Native ARM64 (aarch64) support"  
echo "  ✅ All languages via 'apk add'"
echo "  ✅ Works perfectly in PRoot on Android"
echo "  ✅ iOS devices can connect via WiFi to port 3000/8082"
echo ""

# 1. Install Termux dependencies
echo "📦 Installing proot, wget, tar..."
pkg install -y proot wget tar 2>/dev/null || true

# 2. Create Alpine rootfs directory
echo "📁 Creating Alpine rootfs at $ALPINE_ROOT..."
mkdir -p "$ALPINE_ROOT"

# 3. Download Alpine minirootfs
TARBALL="$HOME/alpine-minirootfs.tar.gz"
if [ ! -f "$TARBALL" ]; then
    echo "⬇️  Downloading Alpine Linux ${ALPINE_VERSION} (${ALPINE_ARCH})..."
    wget -q --show-progress -O "$TARBALL" "$ALPINE_URL"
else
    echo "✅ Alpine tarball already cached, skipping download."
fi

# 4. Extract
echo "📦 Extracting rootfs..."
tar -xzf "$TARBALL" -C "$ALPINE_ROOT"

# 5. DNS + hosts
mkdir -p "$ALPINE_ROOT/etc"
echo -e "nameserver 1.1.1.1\nnameserver 8.8.8.8" > "$ALPINE_ROOT/etc/resolv.conf"
echo -e "127.0.0.1 localhost\n::1 localhost" > "$ALPINE_ROOT/etc/hosts"

# 6. Helper: enter Alpine shell interactively
ENTER_SCRIPT="$HOME/alpine-shell"
cat > "$ENTER_SCRIPT" << EOF
#!/data/data/com.termux/files/usr/bin/bash
exec proot \\
    --rootfs="$ALPINE_ROOT" \\
    --bind=/dev --bind=/proc --bind=/sys \\
    --bind=/data/data/com.termux/files/home \\
    --bind=/sdcard \\
    -0 \\
    /usr/bin/env -i \\
    HOME=/root TERM="\$TERM" \\
    PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin \\
    LANG=C.UTF-8 \\
    /bin/sh -l "\$@"
EOF
chmod +x "$ENTER_SCRIPT"

# 7. Helper: run codedroid_api inside Alpine (for iOS network access)
API_BIN=""
for candidate in \
    "$HOME/CodeDroid/codedroid_api/target/release/codedroid_api" \
    "$TERMUX_PREFIX/bin/codedroid_api" \
    "$HOME/.local/bin/codedroid_api"; do
    if [ -f "$candidate" ]; then
        API_BIN="$candidate"
        break
    fi
done

START_SCRIPT="$HOME/start-codedroid"
cat > "$START_SCRIPT" << EOF
#!/data/data/com.termux/files/usr/bin/bash
# Start codedroid_api inside Alpine PRoot
# iOS devices can connect to http://\$(ip addr show wlan0 | grep "inet " | awk '{print \$2}' | cut -d/ -f1):3000

API_BIN="$API_BIN"
ALPINE_ROOT="$ALPINE_ROOT"

if [ -z "\$API_BIN" ] || [ ! -f "\$API_BIN" ]; then
    echo "❌ codedroid_api binary not found."
    echo "   Build it first: cd ~/CodeDroid/codedroid_api && cargo build --release"
    exit 1
fi

echo "🚀 Starting codedroid_api inside Alpine PRoot..."
PHONE_IP=\$(ip addr show wlan0 2>/dev/null | grep "inet " | awk '{print \$2}' | cut -d/ -f1)
echo "📱 iOS devices: connect to http://\${PHONE_IP:-<your-phone-ip>}:3000"

exec proot \\
    --rootfs="\$ALPINE_ROOT" \\
    --bind=/dev --bind=/proc --bind=/sys \\
    --bind=/data/data/com.termux/files/home \\
    --bind=/sdcard \\
    -0 \\
    "\$API_BIN"
EOF
chmod +x "$START_SCRIPT"

# 8. Initial Alpine setup
echo ""
echo "🔄 Initialising Alpine apk (this takes ~1 min)..."
proot \
    --rootfs="$ALPINE_ROOT" \
    --bind=/dev --bind=/proc --bind=/sys -0 \
    /bin/sh -c "
        apk update && \
        apk add --no-cache \
            build-base bash curl wget git \
            ca-certificates openssl \
            shadow procps vim nano
    "

echo ""
echo "╔══════════════════════════════════════════════════════════╗"
echo "║  ✅  Alpine Linux PRoot setup complete!                  ║"
echo "╠══════════════════════════════════════════════════════════╣"
echo "║  Rootfs:          ~/alpine/                              ║"
echo "║  Enter shell:     bash ~/alpine-shell                    ║"
echo "║  Start API:       bash ~/start-codedroid                 ║"
echo "║                                                          ║"
echo "║  iOS/Remote:      http://<phone-ip>:3000                 ║"
echo "║  Language Hub:    Open CodeDroid app → badge (top-right) ║"
echo "╚══════════════════════════════════════════════════════════╝"
echo ""

# Show phone IP for iOS users
PHONE_IP=$(ip addr show wlan0 2>/dev/null | grep "inet " | awk '{print $2}' | cut -d/ -f1 || echo "unknown")
echo "📱 Your phone IP: $PHONE_IP"
echo "   Tell iOS users to visit: http://$PHONE_IP:3000"
