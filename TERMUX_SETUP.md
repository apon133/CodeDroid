# 📱 Running CodeDroid on Mobile (Termux)

CodeDroid is highly optimized and can run directly on your Android device using **Termux**. This guide will help you set up the API server, the Intelligent Code Suggestions (LSP), and the built-in Web IDE.

---

## 📥 Step 1: Download & Install Termux

> [!IMPORTANT]
> **DO NOT** download Termux from the Google Play Store, as it is outdated and no longer receives updates.

1. Go to the [Termux F-Droid page](https://f-droid.org/en/packages/com.termux/).
2. Scroll down to the **Packages** section and click **Download APK**.
3. Install the APK (you may need to allow "Unknown Sources" in your phone settings).

---

## 🛠️ Step 2: Environment Setup

Once Termux is installed, run the following commands to set up the base environment:

1. **Update packages**:
   ```bash
   pkg update && pkg upgrade
   ```

2. **Install core dependencies**:
   ```bash
   pkg install rust clang build-essential git nodejs-lts python
   ```

3. **Install build tools for the Web IDE**:
   ```bash
   rustup target add wasm32-unknown-unknown
   ```

---

## 🧠 Step 3: Install Language Servers (IntelliSense)

To get real-time code completions, you need to install the language servers for the languages you use:

- **Rust**: `pkg install rust-analyzer`
- **Python**: `pip install python-lsp-server`
- **C/C++**: `pkg install clangd`
- **Go**: `pkg install gopls`
- **JS/TS**: `npm install -g typescript-language-server typescript`
- **Dart**: Included with the Dart SDK (`pkg install dart`)

---

## 🚀 Step 4: Clone & Run

1. **Clone the repository**:
   ```bash
   git clone https://github.com/apon133/CodeDroid.git
   cd CodeDroid
   ```

2. **Run the API Server**:
   ```bash
   cd codedroid_api
   cargo run --release
   ```
   *The API server will start and wait for connections.*

3. **Start Coding**:
   Visit **[codedroid.netlify.app](https://codedroid.netlify.app)** in your mobile browser and connect it to your local server to start coding!

---

## 📦 Additional Languages

Install the runtimes for the languages you want to execute:

| Language | Installation Command |
|---|---|
| **Dart** | `pkg install dart` |
| **Go** | `pkg install golang` |
| **Java** | `pkg install openjdk-17` |
| **Kotlin** | `pkg install kotlin` |
| **Swift** | `pkg install swift` |
| **Ruby** | `pkg install ruby` |
| **C#** | `pkg install dotnet-sdk` |

---

## 🌐 Modern JS Frameworks

The CodeDroid IDE automatically supports React, Vue, Svelte, and Next.js. Just ensure **Node.js** is installed:

1. `pkg install nodejs-lts`
2. Create a project folder and CodeDroid will handle the rest!
