# CodeDroid — Mobile Code Execution Engine for Android & iOS

<p align="center">
  <img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" alt="Rust">
  <img src="https://img.shields.io/badge/Android-3DDC84?style=for-the-badge&logo=android&logoColor=white" alt="Android">
  <img src="https://img.shields.io/badge/iOS-000000?style=for-the-badge&logo=apple&logoColor=white" alt="iOS">
  <img src="https://img.shields.io/badge/Web-4285F4?style=for-the-badge&logo=googlechrome&logoColor=white" alt="Web">
  <img src="https://img.shields.io/badge/WebAssembly-654FF0?style=for-the-badge&logo=webassembly&logoColor=white" alt="WASM">
  <img src="https://img.shields.io/badge/License-GPLv3-blue?style=for-the-badge" alt="License: GPL v3">
</p>
<p align="center">
  <img src="https://img.shields.io/github/stars/apon133/CodeDroid?style=for-the-badge" alt="GitHub Stars">
  <img src="https://img.shields.io/github/forks/apon133/CodeDroid?style=for-the-badge" alt="GitHub Forks">
  <img src="https://img.shields.io/github/issues/apon133/CodeDroid?style=for-the-badge" alt="GitHub Issues">
  <img src="https://img.shields.io/github/last-commit/apon133/CodeDroid?style=for-the-badge" alt="Last Commit">
  <img src="https://img.shields.io/badge/PRs-welcome-brightgreen?style=for-the-badge" alt="PRs Welcome">
</p>

> **Free, open-source mobile IDE and code execution engine** — write and run Python, Rust, Go, JavaScript, Java, C++, and 13+ languages directly on your Android or iOS device. No laptop needed.

**[🌐 Live Demo](https://codedroid.netlify.app)** · **[📖 Termux Setup](./TERMUX_SETUP.md)** · **[🤝 Contributing](./CONTRIBUTING.md)**

---

## What is CodeDroid?

CodeDroid is a **mobile programming environment** built for developers who code everywhere. Under the hood, it's a high-performance HTTP API server written in **Rust (Axum)** that communicates with your device's actual compilers and runtimes — not a sandbox, not a toy.

It pairs with an integrated **Leptos-based Web IDE** (compiled to WASM) that delivers a desktop-class coding experience on mobile. You get IntelliSense, real package managers, and even web server previews — all running on your phone via **Termux**.

---

## ✨ Features

- **Real execution** — runs your code using actual system compilers (`rustc`, `gcc`, `python3`, `go`). Real output, real errors.
- **LSP-powered IntelliSense** — real-time code completions, error highlighting, and hover documentation via language servers running on-device.
- **Full package manager support** — `pip install`, `cargo add`, `npm install` — dependency installation handled automatically before execution.
- **Web project support** — detects the dev server URL from output logs for instant React, Vue, and Vite previews in the built-in browser.
- **13+ supported languages** — Python, Rust, Go, JavaScript/TypeScript, C/C++, Dart, Java, Kotlin, Swift, C#, Ruby, and more.
- **Works offline** — the API server runs entirely on your device.
- **Cross-platform** — Android (Termux), Linux, macOS, Windows.

---

## 🛠️ Supported Languages & IntelliSense

| Language | Runtime | Package Manager | LSP / IntelliSense |
|---|---|---|---|
| [**Rust**](./docs/languages/rust.md) | `cargo` / `rustc` | `cargo` | ✅ `rust-analyzer` |
| [**Python**](./docs/languages/python.md) | `python3` | `pip3` | ✅ `pylsp` |
| [**Go**](./docs/languages/go.md) | `go run` | `go get` | ✅ `gopls` |
| [**JavaScript**](./docs/languages/javascript.md) / [**TS**](./docs/languages/typescript.md) | `node` / `tsx` | `npm` | ✅ `typescript-language-server` |
| [**C**](./docs/languages/c.md) / [**C++**](./docs/languages/cpp.md) | `gcc` / `g++` / `clang` | `pkg install` | ✅ `clangd` |
| [**Dart**](./docs/languages/dart.md) | `dart` | `pub` | ✅ `dart language-server` |
| [**Java**](./docs/languages/java.md) | `javac` + `java` | Maven | ✅ `jdtls` |
| [**Kotlin**](./docs/languages/kotlin.md) | `kotlinc` | — | ✅ `kotlin-language-server` |
| [**Swift**](./docs/languages/swift.md) | `swift` | SPM | ✅ `sourcekit-lsp` |
| [**C#**](./docs/languages/csharp.md) | `dotnet` | `nuget` | — |
| [**Ruby**](./docs/languages/ruby.md) | `ruby` | `gem` | ✅ `solargraph` |
| [**R**](./docs/languages/r.md) | `Rscript` | — | — |
| [**Scala**](./docs/languages/scala.md) | `scala` | — | — |
| [**Perl**](./docs/languages/perl.md) | `perl` | — | — |
| [**Haskell**](./docs/languages/haskell.md) | `runhaskell` | — | — |
| [**Pascal**](./docs/languages/pascal.md) | `fpc` | — | — |


---

## 🚀 Getting Started

### Prerequisites

- Android device with [Termux](https://termux.dev) installed, **or** a Linux/macOS/Windows machine.
- For the hosted Web IDE: any modern browser at **[codedroid.netlify.app](https://codedroid.netlify.app)**.

### Mobile Setup (Android / Termux)

Full step-by-step instructions: 👉 **[TERMUX_SETUP.md](./TERMUX_SETUP.md)**

---

## 📡 API Reference

CodeDroid exposes a simple HTTP API. All endpoints accept and return JSON.

### `POST /run` — Execute Code

Run code in any supported language. Returns `stdout`, `stderr`, and (for web projects) the live server URL.

### `POST /complete` — Get Code Completions

Returns LSP-powered code suggestions for the given cursor position.

```json
{
  "code": "fn main() { pri",
  "language": "rust",
  "project_path": "/home/my_project",
  "line": 0,
  "character": 15
}
```

### `POST /sync_file` — Sync File to Disk

Creates or updates a file on the device. Required for LSP and multi-file projects.

### `POST /stop` — Stop a Running Process

Kills a long-running process (e.g., a dev server) by PID.

---

## 🏗️ Architecture

```
       [ Web IDE — Leptos / WASM ]
                   │
           HTTP / JSON Requests
                   ▼
    [ CodeDroid API Server — Rust / Axum ]
          │                   │
    [ LSP Servers ]     [ System Runtimes ]
  (rust-analyzer,        (python3, cargo,
   clangd, gopls…)        gcc, node, go…)
```

---

## 💻 Tech Stack

| Layer | Technology |
|---|---|
| API Server | [Rust (Axum)](./codedroid_api/README.md) |
| Web IDE | [Leptos (WASM)](./codedroid_frontend/README.md) |
| IntelliSense | LSP (Language Server Protocol) |
| Runtime | Termux (Android), Linux, macOS, Windows |

---

## 🤝 Contributing

Contributions are welcome. Please read **[CONTRIBUTING.md](./CONTRIBUTING.md)** for guidelines on reporting bugs, suggesting features, and submitting pull requests.

---

## 📄 License

GNU General Public License v3.0 — see [LICENSE](LICENSE) for full terms.

---

## 👤 Author

**Md Apon Ahmed**
GitHub: [@apon133](https://github.com/apon133)

---

*CodeDroid — Because real developers code everywhere.*