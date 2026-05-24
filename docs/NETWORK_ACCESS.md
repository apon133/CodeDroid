# 📱 iOS & Remote Device Setup Guide

Use CodeDroid on **any device** (iPhone, iPad, another Android, tablet) by connecting to a backend running on your PC or Android phone over your local WiFi network.

---

## 🏗️ How It Works

```
┌─────────────────────────┐         ┌──────────────────────────────┐
│   iOS / iPad / Browser  │ ──WiFi─▶│  PC / Android (Termux)       │
│                         │         │                              │
│  Open in Safari/Chrome: │         │  codedroid_api  → port 3000  │
│  http://<IP>:8082       │         │  trunk serve    → port 8082  │
└─────────────────────────┘         └──────────────────────────────┘
```

Both devices **must be on the same WiFi network.**

---

## 🖥️ Step 1: Start the Backend (on PC / Android)

### On PC (macOS / Linux)
```bash
# Find your local IP first
ipconfig getifaddr en0   # macOS WiFi
# or
hostname -I              # Linux

# Start the API server (listens on all interfaces automatically)
cd codedroid_api
cargo run --release

# Start the frontend server with network access
cd codedroid_frontend
trunk serve --port 8082 --address 0.0.0.0
```

### On Android (Termux)
```bash
# Find your phone's local IP
ip addr show wlan0 | grep "inet "

# Start the API
cd ~/codedroid_api
cargo run --release

# Start the frontend
cd ~/codedroid_frontend
trunk serve --port 8082 --address 0.0.0.0
```

---

## 🔍 Step 2: Find Your Device's Local IP

| Platform | Command |
|---|---|
| macOS | `ipconfig getifaddr en0` |
| Linux | `hostname -I` |
| Android (Termux) | `ip addr show wlan0` |
| Windows | `ipconfig` → look for IPv4 Address |

Example output: `192.168.0.101`

---

## 📱 Step 3: Open on iOS / Remote Device

Open **Safari** (or any browser) and navigate to:

```
http://192.168.0.101:8082
```

> Replace `192.168.0.101` with your actual local IP from Step 2.

---

## ⚙️ Step 4: Set Backend URL in Settings

The frontend needs to know where the API is. Inside the app:

1. Tap the **⚙️ Settings** icon
2. Scroll to **🌐 Backend Server**
3. Enter your API URL:
   ```
   http://192.168.0.101:3000
   ```
4. Tap **"Test"** — you should see **✅ Connected!**

> This URL is saved in your browser's LocalStorage — you only need to set it once.

---

## ✅ Verify Everything is Working

From the **server machine**, run:
```bash
# Check both ports are listening on all interfaces
curl http://192.168.0.101:3000/ping   # should return: pong
curl http://192.168.0.101:8082        # should return: HTML page
```

---

## 🔥 Firewall Troubleshooting

If the device **cannot connect**, the firewall may be blocking ports 3000 and 8082.

### macOS Firewall
```
System Settings → Privacy & Security → Firewall → Firewall Options
→ Allow "codedroid_api" and "trunk"
```
Or temporarily disable the firewall for testing:
```bash
sudo /usr/libexec/ApplicationFirewall/socketfilterfw --setglobalstate off
```

### Linux (ufw)
```bash
sudo ufw allow 3000
sudo ufw allow 8082
```

### Android (Termux)
Termux does not have a firewall — ports are open by default once the server is running.

---

## 💡 Tips

- **Same WiFi required** — both devices must be on the same router/network.
- **IP can change** — home routers sometimes assign a new IP after reconnecting. Check it again if the connection stops working.
- **Static IP (optional)** — assign a static local IP to your server device in your router settings so the IP never changes.
- **HTTPS not required** — since both devices are on a local network, plain `http://` works fine in browsers.

---

## 📋 Quick Reference

| Service | Local only | Network access |
|---|---|---|
| Frontend | `http://localhost:8082` | `http://<your-ip>:8082` |
| Backend API | `http://localhost:3000` | `http://<your-ip>:3000` |
| Ping check | `http://localhost:3000/ping` | `http://<your-ip>:3000/ping` |
