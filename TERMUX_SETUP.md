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

### 🚀 One-Line Setup & Run

```bash
pkg update -y && pkg upgrade -y && pkg install -y git rust && git clone https://github.com/apon133/CodeDroid.git && cd CodeDroid/codedroid_api && cargo run --release
```

The API server will start and wait for connections.

### **OR**

Once Termux is installed, run the following commands to set up the base environment:

1. **Update packages**:
   ```bash
   pkg update && pkg upgrade
   ```

2. **Install core dependencies**:
   ```bash
   pkg install rust clang build-essential git
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

## 📦 Language Setup & Auto-suggestions Guides

For detailed, step-by-step mobile environment setup and configuration instructions with auto-suggestions, click on your language below:

| Language | Language Server / LSP | Detailed Guide |
|---|---|---|
| **Rust** | `rust-analyzer` | 👉 [Rust Setup Guide](./docs/languages/rust.md) |
| **Go** | `gopls` | 👉 [Go Setup Guide](./docs/languages/go.md) |
| **Dart** | `dart-language-server` | 👉 [Dart Setup Guide](./docs/languages/dart.md) |
| **C** | `clangd` | 👉 [C Setup Guide](./docs/languages/c.md) |
| **C++** | `clangd` | 👉 [C++ Setup Guide](./docs/languages/cpp.md) |
| **C#** | `dotnet` | 👉 [C# Setup Guide](./docs/languages/csharp.md) |
| **Java** | `jdtls` | 👉 [Java Setup Guide](./docs/languages/java.md) |
| **Python** | `pylsp` | 👉 [Python Setup Guide](./docs/languages/python.md) |
| **Kotlin** | `kotlin-language-server` | 👉 [Kotlin Setup Guide](./docs/languages/kotlin.md) |
| **Swift** | `sourcekit-lsp` | 👉 [Swift Setup Guide](./docs/languages/swift.md) |
| **Ruby** | `solargraph` | 👉 [Ruby Setup Guide](./docs/languages/ruby.md) |
| **JavaScript** | `typescript-language-server` | 👉 [JavaScript Setup Guide](./docs/languages/javascript.md) |
| **TypeScript** | `typescript-language-server` | 👉 [TypeScript Setup Guide](./docs/languages/typescript.md) |
| **R** | `Rscript` | 👉 [R Setup Guide](./docs/languages/r.md) |
| **Scala** | `scalac` | 👉 [Scala Setup Guide](./docs/languages/scala.md) |
| **Perl** | `perl` | 👉 [Perl Setup Guide](./docs/languages/perl.md) |
| **Haskell** | `runhaskell` | 👉 [Haskell Setup Guide](./docs/languages/haskell.md) |
| **Pascal** | `fpc` | 👉 [Pascal Setup Guide](./docs/languages/pascal.md) |


---

## 🌐 Modern JS Frameworks

The CodeDroid IDE automatically supports React, Vue, Svelte, and Next.js. Just ensure **Node.js** is installed:

1. `pkg install nodejs-lts`
2. Create a project folder and CodeDroid will handle the rest!
