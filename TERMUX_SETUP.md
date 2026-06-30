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

## ⚡ Step 2: Quick & Easy Installation (No Git or Compilation)

The fastest way to install or update CodeDroid on Termux is with our single-line installer. It detects your device architecture (ARM64 or x86_64), downloads the precompiled binary, and registers a global `codedroid` command.

Run this command in Termux:

```bash
curl -sL https://raw.githubusercontent.com/apon133/CodeDroid/main/install.sh | bash
```

Once installed, you can manage CodeDroid with the following commands:
- **Start the API Server**: `codedroid` or `codedroid start`
- **Update to Latest Version**: `codedroid update`
- **Uninstall**: `codedroid uninstall`

*This method runs the pre-compiled binary directly and does not require git or installing Rust.*

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

## 🛠️ Alternate Step 4: Clone & Run (For Developers)

If you prefer to clone the repository and compile the API from source:

1. **Install dependencies**:
   ```bash
   pkg update && pkg upgrade
   pkg install rust clang build-essential git
   ```

2. **Clone the repository**:
   ```bash
   git clone https://github.com/apon133/CodeDroid.git
   cd CodeDroid
   ```

3. **Run the API Server from source**:
   ```bash
   cd codedroid_api
   cargo run --release
   ```

   Or to compile and run the local binary:
   ```bash
   ./run.sh
   ```

4. **Start Coding**:
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
