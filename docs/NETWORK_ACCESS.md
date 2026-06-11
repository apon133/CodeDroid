# 📱 CodeDroid — iOS & Remote Device Setup Guide

> Use CodeDroid on **any device** — iPhone, iPad, another Android phone, or tablet — by connecting to a backend server running on your PC or Android phone over your local WiFi network.

---

## 🧠 How It Works (Simple Explanation)

Think of it like a **two-device teamwork setup**:

- **Device A (your PC or Android phone)** runs the actual code execution engine in the background.
- **Device B (your iPhone, iPad, or any browser device)** opens the CodeDroid app as a webpage and sends code to Device A to run.

They talk to each other over your home WiFi — no internet needed.

```
┌──────────────────────────┐         ┌────────────────────────────────┐
│  iPhone / iPad / Browser │──WiFi──▶│  PC or Android (Termux)        │
│                          │         │                                │
│  Open in Safari/Chrome:  │         │  codedroid_api  → port 3000    │
│  http://<YOUR-IP>:8082   │         │  trunk serve    → port 8082    │
└──────────────────────────┘         └────────────────────────────────┘
```

> 💡 **Both devices must be connected to the same WiFi network** — like your home router.

---

## ✅ Prerequisites

Before starting, make sure you have the following installed on your **PC or Android phone**:

- [Rust + Cargo](https://rustup.rs/) — to build and run `codedroid_api`
- [Trunk](https://trunkrs.dev/) — to serve the frontend (`cargo install trunk`)
- The CodeDroid project cloned and ready

---

## 🚀 Step-by-Step Setup

### Step 1 — Start the Backend (on your PC or Android)

This starts the code execution engine so other devices can send code to it.

#### On PC (macOS or Linux)

**Option A: Pre-compiled Binary (Direct Run)**
If you have a pre-compiled `codedroid-api` binary in the root directory:
```bash
# Start the API server directly
./run.sh
# OR run the binary directly:
./codedroid-api
```

**Option B: Build from Source (Requires Rust/Cargo)**
```bash
# Navigate to the API folder
cd codedroid_api

# Start the API server
cargo run --release
```

**Option C: Compile & Copy Binary to Root (For Developers)**
```bash
# Compile and place binary in the root
./run.sh --build
```

Then in a **new terminal window**, start the frontend:

```bash
# Navigate to the frontend folder
cd codedroid_frontend

# Serve it on your local network (important: use --address 0.0.0.0)
trunk serve --port 8082 --address 0.0.0.0
```

#### On Android (Termux)

**Option A: Pre-compiled Binary (Direct Run)**
If you have a pre-compiled `codedroid-api` binary in the root directory:
```bash
# Start the API server directly
./run.sh
# OR run the binary directly:
./codedroid-api
```

**Option B: Build from Source (Requires Rust/Cargo)**
```bash
# Navigate to the API folder
cd ~/codedroid_api

# Start the API server
cargo run --release
```

**Option C: Compile & Copy Binary to Root (For Developers)**
```bash
# Compile and place binary in the root
./run.sh --build
```

Open a **new Termux session** and run:

```bash
cd ~/codedroid_frontend
trunk serve --port 8082 --address 0.0.0.0
```

> ⚠️ The `--address 0.0.0.0` flag is **required** — without it, other devices on your network won't be able to reach the frontend.

---

### Step 2 — Find Your Device's Local IP Address

Your "local IP" is the address other devices on your network use to find your PC/phone. It looks like `192.168.x.x`.

| Your Platform | Run This Command |
|---|---|
| macOS | `ipconfig getifaddr en0` |
| Linux | `hostname -I` |
| Android (Termux) | `ip addr show wlan0 \| grep "inet "` |
| Windows | Open CMD → type `ipconfig` → look for **IPv4 Address** |

**Example output:** `192.168.0.101`

Write this IP down — you'll need it in the next steps.

---

### Step 3 — Open CodeDroid on Your iOS / Remote Device

On your iPhone, iPad, or any other device:

1. Open **Safari** (or Chrome, Firefox — any browser works)
2. In the address bar, type:

```
http://192.168.0.101:8082
```

> 🔁 Replace `192.168.0.101` with **your actual IP** from Step 2.

You should see the CodeDroid interface load in your browser.

---

### Step 4 — Connect the App to Your Backend

The app needs to know where your code execution server is running. Here's how to set it:

1. Tap the **⚙️ Settings** icon inside the app
2. Scroll down to **🌐 Backend Server**
3. Enter the API URL (same IP, but port 3000):

```
http://192.168.0.101:3000
```

4. Tap **"Test"**
5. If everything is working, you'll see **✅ Connected!**

> 💾 This setting is saved automatically in your browser — you only need to do this once.

---

## 🔎 Verify Everything Is Working

From your **server machine**, run these two checks:

```bash
# Check that the API is running and reachable
curl http://192.168.0.101:3000/ping
# Expected response: pong

# Check that the frontend is reachable
curl http://192.168.0.101:8082
# Expected response: an HTML page
```

If both return output, your setup is complete. ✅

---

## 🔥 Troubleshooting — Can't Connect?

The most common cause is a **firewall** blocking the ports. Here's how to fix it:

### macOS Firewall

Go to:
```
System Settings → Privacy & Security → Firewall → Firewall Options
→ Allow "codedroid_api" and "trunk"
```

Or temporarily disable the firewall to test:
```bash
sudo /usr/libexec/ApplicationFirewall/socketfilterfw --setglobalstate off
```

### Linux (ufw)

```bash
sudo ufw allow 3000
sudo ufw allow 8082
```

### Android (Termux)

No action needed — Termux has no firewall. Ports are open as soon as the server starts running.

---

## 💡 Tips & Common Issues

| Problem | Solution |
|---|---|
| App won't load on remote device | Make sure both devices are on the **same WiFi** |
| IP address stopped working | Your router may have assigned a new IP — re-run Step 2 |
| "Test" shows ❌ not connected | Double-check the IP and port (3000) in Settings |
| Want a permanent IP | Set a **static local IP** in your router settings for the server device |
| Do I need HTTPS? | No — plain `http://` is fine on a local network |

---

## 📋 Quick Reference

| Service | Local Access | Network Access (replace IP) |
|---|---|---|
| Frontend (app UI) | `http://localhost:8082` | `http://<your-ip>:8082` |
| Backend API | `http://localhost:3000` | `http://<your-ip>:3000` |
| Ping check | `http://localhost:3000/ping` | `http://<your-ip>:3000/ping` |