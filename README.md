# CodeDroid — Run Code on Your Phone 📱

> **The best mobile code execution engine and IDE for Web, Android, iOS, and desktop.**  
> Write and run Python, Rust, Go, JavaScript, Java, C++, and 13+ languages — directly from your phone with real-time IntelliSense.

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](LICENSE)
[![Built with Rust](https://img.shields.io/badge/built%20with-Rust-orange.svg)](https://www.rust-lang.org/)
[![Android Support](https://img.shields.io/badge/Android-Termux%20Ready-green.svg)](./TERMUX_SETUP.md)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](https://github.com/apon133/CodeDroid/pulls)

---

## What is CodeDroid?

**CodeDroid** is a free, open-source **mobile code editor and execution engine** that lets you write and run real code on your Android or iOS device — no laptop needed.

Under the hood, it's a high-performance API server written in **Rust** that talks to your device's compilers and runtimes. It now features an integrated **Leptos-based Web IDE** (WASM) that provides a desktop-class coding experience on mobile. You can use the hosted version at **[codedroid.netlify.app](https://codedroid.netlify.app)**. You get a complete **mobile programming environment** that feels fast, works offline, and supports serious projects with real package managers and **LSP-powered code suggestions**.

---

## Why CodeDroid?

| Problem | CodeDroid's Solution |
|---|---|
| "I can't install a compiler on my phone" | Runs on Termux — full Linux environment on Android |
| "Other mobile IDEs are too slow" | API server written in Rust — near-zero overhead |
| "No IntelliSense on mobile" | **Integrated LSP support** (rust-analyzer, clangd, gopls, etc.) |
| "No package manager support" | `pip install`, `cargo add`, `npm install` — all supported |
| "I need to run a web server from my phone" | Built-in support for Vite, React, Vue dev servers |

---

## Supported Languages & IntelliSense

CodeDroid supports **13+ programming languages** with real-time code suggestions for major ones.

| Language | How It Runs | Package Manager | LSP Support (IntelliSense) |
|---|---|---|---|
| **Rust** | `cargo` / `rustc` | `cargo` | ✅ `rust-analyzer` |
| **Python** | `python3` | `pip3` | ✅ `pylsp` |
| **Go** | `go run` | `go get` | ✅ `gopls` |
| **JavaScript/TS** | `node` / `tsx` | `npm` | ✅ `typescript-language-server` |
| **C / C++** | `gcc` / `g++` | `pkg install` | ✅ `clangd` |
| **Dart** | `dart` | `pub` | ✅ `dart language-server` |
| **Java** | `javac` + `java` | Maven | ✅ |
| **Kotlin** | `kotlinc` | — | — |
| **Swift** | `swift` | SPM | — |
| **C#** | `dotnet` | `nuget` | — |
| **Ruby** | `ruby` | `gem` | — |

---

## Key Features

### ⚡ Real Execution, Not a Sandbox Toy
CodeDroid runs your code using actual system compilers — `rustc`, `gcc`, `python3`, `go`. No fake interpreters. Real output, real errors.

### 🧠 Intelligent Code Suggestions (LSP)
CodeDroid isn't just a text editor. It provides **real-time code completions, error highlighting, and hover information** by running language servers directly on your phone.

### 📦 Full Package Manager Support
Need a library? Just ask CodeDroid to install it. The API handles dependency installation automatically before execution.

### 🖥️ Web Project Support
Running a React, Vue, or Vite project? CodeDroid detects the dev server URL from the output logs and allows you to preview your site instantly in the built-in browser.

---

## API Reference

The CodeDroid API is a simple HTTP server. Here are the main endpoints:

### `POST /run` — Execute Code
Run any supported language. Returns stdout, stderr, and (for web projects) the server URL.

### `POST /complete` — Get Code Suggestions
Get intelligent code completions using LSP servers.

**Request:**
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
Updates or creates a file on the device. Essential for LSP and multi-file projects.

### `POST /stop` — Stop a Running Process
Kill a long-running process (like a dev server) by PID.

---

## Technical Architecture

```
       [ Modern Web IDE (Leptos/WASM) ]
                   │
           HTTP / JSON Requests
                   ▼
    [ CodeDroid API Server (Rust/Axum) ]
          │                │
    [ LSP Servers ]  [ System Runtimes ]
    (rust-analyzer,   (python, cargo,
     clangd, etc.)     gcc, node, etc.)
```

---

## Tech Stack

- **Backend:** [Rust (Axum)](./codedroid_api/README.md)
- **Frontend:** [Leptos (WASM)](./codedroid_frontend/README.md)
- **IntelliSense:** LSP (Language Server Protocol)
- **Deployment:** Termux (Android), Linux, macOS, Windows

---

## Mobile Setup (Android / Termux)

Full step-by-step guide: 👉 **[TERMUX_SETUP.md](./TERMUX_SETUP.md)**

---

## Contributing

All contributions are welcome! Please see our **[CONTRIBUTING.md](./CONTRIBUTING.md)** for guidelines on how to report bugs, suggest features, and submit code changes.

---

## License

GNU General Public License v3.0 — see [LICENSE](LICENSE) for details.

---

## Contact

**Md Apon Ahmed**  
GitHub: [@apon133](https://github.com/apon133)  

---

*CodeDroid — Because real developers code everywhere.*