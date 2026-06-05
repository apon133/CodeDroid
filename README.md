# CodeDroid — Mobile Code Execution Engine for Android & iOS

<p align="center">
  <img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" alt="Rust">
  <img src="https://img.shields.io/badge/Leptos-EF3939?style=for-the-badge&logo=leptos&logoColor=white" alt="Leptos">
  <img src="https://img.shields.io/badge/Axum-333333?style=for-the-badge&logo=rust&logoColor=white" alt="Axum">
  <img src="https://img.shields.io/badge/WebAssembly-654FF0?style=for-the-badge&logo=webassembly&logoColor=white" alt="WASM">
  <img src="https://img.shields.io/badge/Tauri-24C8D8?style=for-the-badge&logo=tauri&logoColor=white" alt="Tauri">
  <img src="https://img.shields.io/badge/Flutter-02569B?style=for-the-badge&logo=flutter&logoColor=white" alt="Flutter">
</p>
<p align="center">
  <img src="https://img.shields.io/badge/Android-3DDC84?style=for-the-badge&logo=android&logoColor=white" alt="Android">
  <img src="https://img.shields.io/badge/iOS-000000?style=for-the-badge&logo=apple&logoColor=white" alt="iOS">
  <img src="https://img.shields.io/badge/Termux-000000?style=for-the-badge&logo=terminal&logoColor=white" alt="Termux">
  <img src="https://img.shields.io/badge/Web-4285F4?style=for-the-badge&logo=googlechrome&logoColor=white" alt="Web">
  <img src="https://img.shields.io/badge/Git-F05032?style=for-the-badge&logo=git&logoColor=white" alt="Git">
  <img src="https://img.shields.io/badge/License-GPLv3-blue?style=for-the-badge" alt="License: GPL v3">
</p>
<p align="center">
  <img src="https://img.shields.io/github/stars/apon133/CodeDroid?style=for-the-badge" alt="GitHub Stars">
  <img src="https://img.shields.io/github/forks/apon133/CodeDroid?style=for-the-badge" alt="GitHub Forks">
  <img src="https://img.shields.io/github/issues/apon133/CodeDroid?style=for-the-badge" alt="GitHub Issues">
  <img src="https://img.shields.io/github/last-commit/apon133/CodeDroid?style=for-the-badge" alt="Last Commit">
  <img src="https://img.shields.io/badge/PRs-welcome-brightgreen?style=for-the-badge" alt="PRs Welcome">
  <img src="https://hits.sefy.io/v1?url=https://github.com/apon133/CodeDroid&label=Views&color=pink&style=for-the-badge" alt="Hits Badge">
</p>

> **Free, open-source mobile IDE and code execution engine** — write, compile, run, and debug Python, Rust, Go, JavaScript, Java, C++, and 13+ languages directly on your Android or iOS device. No laptop needed, running with bare-metal performance.

<p align="center">
  <a href="https://codedroid.netlify.app" target="_blank">
    <img src="https://img.shields.io/badge/Live%20Demo-4285F4?style=for-the-badge&logo=googlechrome&logoColor=white" alt="Live Demo">
  </a>
  <a href="./TERMUX_SETUP.md">
    <img src="https://img.shields.io/badge/Termux%20Setup-3DDC84?style=for-the-badge&logo=android&logoColor=white" alt="Termux Setup">
  </a>
  <a href="./CONTRIBUTING.md">
    <img src="https://img.shields.io/badge/Contributing-000000?style=for-the-badge&logo=github&logoColor=white" alt="Contributing">
  </a>
  <a href="https://discord.gg/5srCEqsht" target="_blank">
    <img src="https://img.shields.io/badge/Discord-5865F2?style=for-the-badge&logo=discord&logoColor=white" alt="Discord">
  </a>
  <a href="https://t.me/codedroid133" target="_blank">
    <img src="https://img.shields.io/badge/Telegram-24A1DE?style=for-the-badge&logo=telegram&logoColor=white" alt="Telegram">
  </a>
  <a href="https://www.youtube.com/@CodeDroidYT" target="_blank">
    <img src="https://img.shields.io/badge/YouTube-FF0000?style=for-the-badge&logo=youtube&logoColor=white" alt="YouTube">
  </a>
</p>

---

## 📖 Table of Contents
1. [What is CodeDroid?](#-what-is-codedroid)
2. [Application Preview](#-application-preview)
3. [Key Capabilities & Features](#-key-capabilities--features)
4. [Ecosystem & Architectural Blueprints](#-ecosystem--architectural-blueprints)
5. [Native Wrapper Applications (Tauri & Flutter)](#-native-wrapper-applications-tauri--flutter)
6. [Supported Languages & Setup Directory](#-supported-languages--setup-directory)
7. [Smart Code Suggestions & AI Rule Engine](#-smart-code-suggestions--ai-rule-engine)
8. [API Endpoint Specifications (No Code Payloads)](#-api-endpoint-specifications-no-code-payloads)
9. [Termux Android Installation Details](#-termux-android-installation-details)
10. [Cross-Device & iOS Network Connectivity](#-cross-device--ios-network-connectivity)
11. [LSP Binary Executable Path Resolution](#-lsp-binary-executable-path-resolution)
12. [Upcoming Features & Roadmap](#-upcoming-features--roadmap)
13. [Contributing](#-contributing)
14. [License](#-license)

---

CodeDroid is a high-performance **mobile programming environment** that compiles and runs your code directly on-device with zero virtualization. It is built as three integrated modules:

1. **`codedroid_api` (Backend Engine)**: A lightweight, multi-threaded server written in **Rust (Axum)**. It directly invokes system compilers (`rustc`, `gcc`, `kotlinc`, `javac`, etc.), manages asynchronous execution streams, coordinates multiple concurrent Language Server Protocol (LSP) instances, and handles local dependency installation.
2. **`codedroid_frontend` (Web IDE)**: A reactive, mobile-first IDE built using the **Leptos** web framework and compiled to **WebAssembly (WASM)**. It runs entirely inside any web browser (Safari, Chrome, Firefox) and utilizes high-performance code-rendering pipelines to provide syntax highlighting, bracket matching, file drawers, autocomplete drop-downs, and compiler error overlays.
3. **`apps` (Native Wrappers)**: Cross-platform native wrappers bundling the WebAssembly frontend. Includes **Tauri** for desktop platforms (macOS, Windows, Linux) and **Flutter** for mobile platforms (Android) running a lightweight, local web server background task to bypass browser WebAssembly restrictions.

Unlike typical cloud-based sandboxes or emulated JS runtimes, CodeDroid exposes the *real* filesystem and *real* binaries of your host device (like a Termux Android shell or local macOS/Linux installation). 

---

## 📱 Application Preview

| Create a New Project | Add Dependencies |
| :---: | :---: |
| ![Create Project](./assets/create_project.jpg) | ![Add Dependency](./assets/dependency_add.jpg) |

| Auto Completion & Suggestions | Error Diagnostics | Hover Documentation |
| :---: | :---: | :---: |
| ![Code Suggestion](./assets/code_suggetion.jpg) | ![Error Suggestion](./assets/error_suggetion.jpg) | ![Code Documentation](./assets/code_documention.jpg) |

| In-File Search | Global Project Search |
| :---: | :---: |
| ![In-File Search](./assets/in_file_search.jpg) | ![Global Search](./assets/globel_search.jpg) |

---

## ✨ Key Capabilities & Features

*   **Mobile-First Touch Architecture**: Designed specifically for small touchscreens (320px–768px). Features a slide-out file explorer overlay drawer, auto-collapsing sidebar upon opening files, persistent touch targets for closing tabs, and custom layout controls utilizing CSS `100dvh` to prevent keyboard resize clipping.
*   **Universal Drag-to-Resize Layout**: Dynamically adjust sidebars, bottom terminal consoles, and active editor split panes using intuitive touch-and-drag handle borders. Panel dimensions are persistently saved to `LocalStorage` for continuity across sessions.
*   **Interactive Web Terminal**: Run shell commands directly using a fully connected terminal console (backed by real system PTY shells) with raw stream capturing, exit code detection, and custom termination controls.
*   **Integrated Git Console**: Stage or unstage workspace changes, view line-by-line colored diff overlays directly inside the editor, generate commit messages with AI suggestions, clone repositories, and push/pull updates directly from the IDE sidebar.
*   **Live Reload Development Server**: Spin up a local static server inside any project directory that monitors folder modifications and dynamically injects hot-reloading scripts, bypassing manual browser refresh cycles.
*   **Rich Media & Document Viewers**: Native tab views to render images, video playback, and audio files directly, alongside a side-by-side Markdown layout editor and previewer.
*   **LSP-powered IntelliSense**: Floating completion panels, in-line diagnostics, hover tooltips, definition lookups, and references. Automatically hides bloated metadata subpanels on narrow screens to prevent viewport clipping.
*   **Save-Triggered Synchronization**: Supports immediate file-writing to the host disk on save, triggering automatic LSP change notifications (`textDocument/didChange` & `textDocument/didSave`) which instantly update error diagnostics.
*   **Modern Web Framework Scaffolding**: Bootstrap web apps with optimized templates for React (Vite), Vue (Vite), Svelte (Vite), Next.js (App Router), Remix, and Angular.
*   **Live Web Preview & Cross-Device Refresh**: Automatically scans development server stdout logs (e.g. Vite, Next.js) for local addresses, launches an in-IDE browser viewport with manual/auto reload, and translates local loops (127.0.0.1) into LAN IPs so iOS and tablet clients can access previews.
*   **Advanced File Manager**: Create, copy, paste, delete, move, or rename files and directories. Full recursive operations synchronized instantly to local disk and LocalStorage states.
*   **Persistent Editor State**: Restores open file tabs, split views, active cursor position, and panel visibility structures when reloading.
*   **Offline Fallback Mode**: Works entirely offline with local-first file caching. If the LSP is unavailable, CodeDroid falls back to an internal regex token parser to provide syntax-matching autocomplete suggestions.

---

## 🏗️ Ecosystem & Architectural Blueprints

### Codebase Directory Structures & Module Overview

The CodeDroid architecture divides logical responsibilities across the system components. Below is a structural mapping of the codebase modules and directories:

```
CodeDroid Root
 ├── codedroid_api (Backend Crate)
 │    ├── src
 │    │    ├── main.rs               # Server setup, routing tables, and CORS policies
 │    │    ├── models.rs             # Parameter schema definitions & conversion methods
 │    │    ├── handlers.rs           # Request interceptors mapping inputs to file system and compilers
 │    │    ├── lsp.rs                # Custom JSON-RPC stdin/stdout manager for running LSPs
 │    │    ├── terminal.rs           # PTY shell session lifecycle and process terminal coordinators
 │    │    ├── git.rs                # Local git repository managers & AI commit message analyzers
 │    │    ├── live_server.rs        # Static server watching file system updates & reload injections
 │    │    ├── runner.rs             # Shell executor engine coordinating processes per runtime
 │    │    ├── error_suggestions.rs  # Rule-based diagnostic suggestions compiler
 │    │    ├── diagnostics.rs        # Asynchronous polling coordinator waiting for LSP diagnostics
 │    │    └── utils.rs              # Path, port, IP resolving & string manipulators
 │    └── Cargo.toml                 # Backend rust configurations & dependencies
 │
 ├── codedroid_frontend (Frontend Crate)
 │    ├── src
 │    │    ├── main.rs               # Leptos hydration bootstrapping and entry points
 │    │    ├── store.rs              # LocalStorage reactive store wrappers
 │    │    ├── models.rs             # Client API data contract mirroring models
 │    │    ├── api.rs                # Async web request client wrapping Gloo-Net
 │    │    └── pages
 │    │         ├── home.rs          # Project selection UI, metadata tables, templates builder
 │    │         ├── settings.rs      # Port definitions, custom server URL forms
 │    │         ├── docs.rs          # Reference manuals and language-specific instructions
 │    │         └── editor (Editor Core Page)
 │    │              ├── mod.rs      # Shell drawer, tabs manager, file navigators, previews
 │    │              ├── code_area.rs# Autocomplete dropdowns, syntax triggers, scroll synchronizers
 │    │              ├── components.rs# Resize grid panel handles, footer consoles, layout tabs
 │    │              ├── git_panel.rs# Source control UI, staging, line-diff viewers, AI message inputs
 │    │              ├── agent_panel.rs# Coding copilot assistant interactive conversation drawers
 │    │              ├── search_bar.rs# Project-wide regex finder UI
 │    │              └── utils.rs    # Syntect themes converter mapping styles
 │    └── Cargo.toml                 # Frontend WASM configurations & packages
 │
 └── apps (Native Wrappers Directory)
      ├── flutter_android/            # Flutter app wrapping frontend for Android with localhost server
      ├── tauri_desktop/              # Tauri wrapper bundling frontend for macOS, Windows, Linux
      └── sync_assets.sh              # Asset synchronization builder and compiler script
```

### Flow Diagram: Document Sync & Compilation Lifecycle

```
 ┌──────────────┐         1. Save (Ctrl+S / Save Button)          ┌────────────────┐
 │  Web IDE     │────────────────────────────────────────────────>│  Axum Backend  │
 │  (Client)    │<────────────────────────────────────────────────│  (Server)      │
 └──────────────┘           4. Return JSON diagnostics            └────────────────┘
        │                                                                  │
        │ 2. notify_file_changed()                                         │ 3. Sync to disk
        ▼                                                                  ▼
 ┌──────────────┐                                                 ┌────────────────┐
 │  LSP Client  │────────────────── JSON-RPC ────────────────────>│  Local File Sys│
 │  (Stdio/RPC) │                                                 │  (/sdcard/...) │
 └──────────────┘                                                 └────────────────┘
```

### Flow Diagram: Diagnostic Polling Loops

```
  Client (Web IDE)                  Axum Backend                   Target LSP
         │                               │                              │
         │─── POST /diagnostics ────────>│                              │
         │    (Wait for version update)  │                              │
         │                               │─── didChange / didSave ─────>│
         │                               │                              │
         │                               │◄── publishDiagnostics ───────│
         │                               │    (Async stderr stream)     │
         │                               │                              │
          │◄── Return Diagnostics ────────│                              │
          │    (JSON range & severity)    │                              │
```

---

## 📱 Native Wrapper Applications (Tauri & Flutter)

CodeDroid offers native client wrappers that bundle the `codedroid_frontend` WebAssembly package directly into target app environments.

### 🔄 Asset Synchronization Flow
Whenever the Leptos frontend code is modified, it must be compiled and synchronized to both the Flutter and Tauri directories. An automated build script is provided:

```bash
# Run from repository root
./apps/sync_assets.sh
```

This script builds `codedroid_frontend` in release mode and mirrors the static outputs to both application directories automatically.

### 📱 Flutter (Android) Wrapper
Located in [`apps/flutter_android/`](./apps/flutter_android/). It launches a background `InAppLocalhostServer` on port `8080` to serve the static assets offline.
- **Why Localhost?** Modern mobile WebViews block WebAssembly streaming compile calls if executed over the `file://` protocol. The local server enables WebAssembly and provides cookies/LocalStorage persistence.
- **Theme & Style**: Tailored with a custom Material 3 Dark theme matching the web IDE, featuring matching system bar colors and transparent status integrations.
- For run and build instructions, see the [Flutter Android README](./apps/flutter_android/README.md).

### 💻 Tauri (Desktop) Wrapper
Located in [`apps/tauri_desktop/`](./apps/tauri_desktop/). It uses Tauri to pack the WebAssembly interface into native desktop clients.
- **Ultra-lightweight**: Compiles to under 20MB, utilizing the native system web engines (Webkit2GTK / WebView2) to reduce RAM usage.
- **Cross-Platform**: Supports native macOS, Windows, and Linux window environments.
- For run and build instructions, see the [Tauri Desktop README](./apps/tauri_desktop/README.md).

---

## 📊 Supported Languages & Setup Directory

This guide details exactly how to configure compilers, runtimes, and auto-suggestion language servers (LSP) for each language. All shell operations are to be run in your Termux or system shell.

---

### 🦀 Rust
Get a full, desktop-grade Rust development environment on your phone.
*   **Compiler & Tools**: Install `rust` and `cargo`:
    ```bash
    pkg install rust
    ```
*   **Language Server (LSP)**: Install `rust-analyzer` for real-time completions, diagnostics, and hover hints:
    ```bash
    pkg install rust-analyzer
    ```
*   **Enable Completions**: Start the API server, open any `.rs` file in the IDE, and start typing. CodeDroid automatically hooks into `rust-analyzer`.

---

### 🐍 Python
Set up Python script execution and IntelliSense formatting.
*   **Python Runtime**: Install Python and pip:
    ```bash
    pkg install python
    ```
*   **Language Server (LSP)**: Install `python-lsp-server` (`pylsp`) via pip:
    ```bash
    pip install python-lsp-server
    ```
*   **Usage**: Create any `.py` file. CodeDroid automatically runs completions and highlights syntax.

---

### 🐹 Go
Full Go compilation toolchain and suggestions on-device.
*   **Go Compiler**: Install the Go programming toolset:
    ```bash
    pkg install golang
    ```
*   **Language Server (LSP)**: Install `gopls` (Go Please) to enable completions:
    ```bash
    pkg install gopls
    ```
*   **Usage**: Open any folder containing `go.mod`, edit `.go` files, and completions will populate.

---

### 🌐 JavaScript & TypeScript
Supports Node.js tools, React, Vue, Svelte, and Next.js.
*   **Runtime**: Install Node.js LTS:
    ```bash
    pkg install nodejs-lts
    ```
*   **Language Server (LSP)**: Install the TypeScript LSP globally using npm:
    ```bash
    npm install -g typescript-language-server typescript
    ```
*   **JS Projects**: Create a `jsconfig.json` or `tsconfig.json` at your project root to assist type-inference resolutions.

---

### 🧡 Svelte
Scaffold and edit Svelte/Vite templates with custom diagnostics.
*   **Language Server (LSP)**: Install Svelte tools:
    ```bash
    npm install -g svelte-language-server typescript
    ```
*   **Usage**: Open any `.svelte` file inside a Vite-scaffolded directory for autocomplete in markup, `<script>`, and `<style>` blocks.

---

### 💚 Vue
Support Vue 3 SFC files.
*   **Language Server (LSP)**: Install Volar Vue language tools:
    ```bash
    npm install -g @vue/language-server typescript
    ```
*   **Usage**: Create or open a `.vue` project. Hybrid type resolutions are managed automatically by the backend.

---

### ☕ Java
Compile and run Java class hierarchies.
*   **Java Runtime & Compiler**: Install the OpenJDK package:
    ```bash
    pkg install openjdk-17
    ```
*   **Language Server (LSP)**: Install the Eclipse JDT Language Server (`jdtls`):
    ```bash
    pkg install jdtls
    ```
*   **Usage**: Edit `.java` files; classes compile automatically inside CodeDroid's runner on run.

---

### 🛡️ C & C++
High-performance native coding using LLVM tools.
*   **Compiler Toolchain**: Install LLVM/Clang and make:
    ```bash
    pkg install clang build-essential
    ```
*   **Language Server (LSP)**: Install `clangd` for diagnostics and completions:
    ```bash
    pkg install clangd
    ```
*   **Usage**: Create a `.c` or `.cpp` file. `clangd` acts as the back-end analyzer.

---

### 🎯 Dart
Build Dart CLI programs and scripts.
*   **Runtime & Toolset**: Install Dart SDK:
    ```bash
    pkg install dart
    ```
*   **Language Server (LSP)**: Dart includes its language server inside the SDK. No separate installation required. CodeDroid resolves it automatically.

---

### 💎 Ruby
Execute scripts and resolve Gems.
*   **Ruby Runtime**: Install ruby:
    ```bash
    pkg install ruby
    ```
*   **Language Server (LSP)**: Install the Solargraph gem:
    ```bash
    gem install solargraph
    ```

---

### 🍎 Swift
Develop Swift scripts inside Termux.
*   **Swift Runtime**: Install Swift:
    ```bash
    pkg install swift
    ```
*   **Language Server (LSP)**: Swift includes the `sourcekit-lsp` binary. Ensure Xcode default toolchains are active if hosting on macOS.

---

### 🧬 Kotlin
Run compiled Kotlin bytecode.
*   **Compiler**: Install Kotlin compiler packages:
    ```bash
    pkg install kotlin
    ```
*   **Language Server (LSP)**: Install `kotlin-language-server` from your system package manager.

---

### 🧪 Haskell, R, Perl, Pascal & Scala
Other supported scripting languages compile and run using their default packages:
*   **Haskell**: Run `pkg install ghc` to compile `.hs` scripts.
*   **R**: Run `pkg install r-base` to execute `.R` formulas.
*   **Perl**: Run `pkg install perl` to execute `.pl` files.
*   **Pascal**: Run `pkg install fpc` to compile `.pas` code with the Free Pascal Compiler.
*   **Scala**: Run `pkg install scala` to run JVM-based Scala programs.

---

## 🧠 Smart Code Suggestions & AI Rule Engine

CodeDroid's suggestion engine in `error_suggestions.rs` parses compiler diagnostics and provides contextual explanations and code replacements.

### Suggestions Rules Mapping

| Rule Trigger | Matching Criteria | Code Replacements | Expected Result |
| :--- | :--- | :--- | :--- |
| **Rust E0384 (Mutability)** | `cannot mutate immutable variable`, `cannot assign to immutable` | Adds `mut` after the variable declaration `let` | Variable is marked mutable; compiler error resolves on save. |
| **Rust Unused Variable** | `unused variable` | Prepends an underscore `_` to the identifier | Silences compiler warning flags. |
| **Rust Type mismatch (String)** | `expected String, found &str` | Appends `.to_string()` or `.into()` | Casts string literal to owned String struct. |
| **Rust Borrow String** | `expected &str, found String` | Prepends borrow operator `&` or appends `.as_str()` | Converts owned String reference to sliced slice. |
| **Rust Integer mismatch** | Mismatches of `usize`/`u32`/`i32` | Appends `as usize` or `as _` | Casts number types dynamically. |
| **Rust Missing Imports** | `cannot find type/struct in scope` for collections/sync | Inserts `use std::collections::*` or `use std::sync::*` at Line 0 | Resolves undefined scope structures. |
| **Rust Move Violations** | `cannot move out of shared reference` | Appends `.clone()` | Creates owned duplicate data segment. |
| **Python Indentation** | `IndentationError`, `unexpected indent` | Informational alignment warning | Alerts layout tabs vs spaces anomalies. |
| **Python Scope Resolution**| `NameError: name is not defined` | Spell-checks declared symbols | Identifies typos or missing scope values. |
| **JS / TS Scope Errors** | `cannot find name` | Identifies missing export tags | Flags typos and missing package imports. |

---

## 📡 API Endpoint Specifications (No Code Payloads)

CodeDroid API runs locally on port `3000` (by default) to bridge your browser interface with the device's system compiler toolchains.

---

### `POST /run`
Runs code in the specified runtime, capturing outputs and dev-server addresses.
*   **Inputs**:
    *   `code`: The raw string of code to execute.
    *   `language`: The identifier matching the compiler profile (e.g. `rust`, `python`, `go`).
    *   `project_path`: Path targeting local directory storage.
    *   `cargo_toml`: Optional customization override flags.
*   **Outputs**:
    *   `output`: Captures execution prints and standard stdout logs.
    *   `error`: Captures compiler failures and standard stderr logs.
    *   `pid`: Spawns a background process ID (returns a number if running a live server).
    *   `url`: The local network endpoint IP resolved from dev server logs (Vite, Next, etc.).

---

### `POST /run_command`
Executes an arbitrary shell command in the project directory context.
*   **Inputs**:
    *   `command`: The command string to run (e.g. `npm install`, `cargo test`).
    *   `project_path`: The workspace path where the command executes.
*   **Outputs**:
    *   `output`: The standard output generated by the shell.
    *   `error`: The standard error generated by the shell.
    *   `success`: Boolean indicating if execution was successful.
    *   `pid`: Optional background process ID if the command remains running.

---

### `POST /stop`
Terminates an active runtime process or development server.
*   **Inputs**:
    *   `pid`: The process ID identifier of the running instance.
*   **Outputs**:
    *   `output`: Confirmation string stating process termination details.
    *   `error`: Standard errors if the process fails to terminate.

---

### `POST /sync_file`
Synchronizes in-editor buffer state to physical disk storage.
*   **Inputs**:
    *   `path`: The absolute path where the file should be written.
    *   `content`: The complete text representation of the file.
*   **Outputs**: Returns a blank HTTP 200 OK status on success.

---

### `POST /add_package`
Runs dependency installations and synchronizes configuration files.
*   **Inputs**:
    *   `package`: Name of the dependency/crate/library to download.
    *   `language`: The target runtime language workspace.
    *   `project_path`: Location of the project source.
*   **Outputs**:
    *   `output`: Standard setup stdout logs from package managers (npm, pip, cargo).
    *   `error`: Errors if dependency resolution fails.
    *   `dependency_file_name`: Configuration file updated (e.g. `Cargo.toml`, `package.json`).
    *   `dependency_file_content`: Updated text content of the configuration file.

---

### `POST /complete`
Fetches autocomplete recommendations from the active language server.
*   **Inputs**:
    *   `code`: File buffer content.
    *   `language`: Target workspace language.
    *   `project_path`: Root folder of the project.
    *   `file_path`: Current active file path.
    *   `line`: Cursor row line number (0-indexed).
    *   `character`: Cursor character column position (0-indexed).
*   **Outputs**:
    *   `suggestions`: Array of objects containing completion lists, labels, insertion texts, documentation summaries, and signature details.

---

### `POST /definition`
Finds the location of a symbol's definition.
*   **Inputs**: Identical parameter structure as `/complete`.
*   **Outputs**:
    *   `locations`: Array of ranges and absolute file paths matching the symbol definition.

---

### `POST /references`
Locates all references to a specific symbol in the workspace.
*   **Inputs**: Identical parameter structure as `/complete`.
*   **Outputs**:
    *   `locations`: Array of absolute paths and range details showing where the symbol is referenced.

---

### `POST /hover`
Retrieves markdown documentation tooltips for variables, methods, or structs.
*   **Inputs**: Identical parameter structure as `/complete`.
*   **Outputs**:
    *   `contents`: Markdown documentation block matching the cursor position.
    *   `error`: Errors if hover data is unavailable.

---

### `POST /diagnostics`
Forces file synchronization and fetches static-analysis errors.
*   **Inputs**: Identical parameter structure as `/complete`.
*   **Outputs**:
    *   `diagnostics`: Array of active compiler warnings, syntax errors, line ranges, severity grades, and compiler codes.

---

### `POST /error_suggestions`
Analyzes a diagnostic payload and suggests quick-fixes.
*   **Inputs**:
    *   `code`: Raw file code string.
    *   `language`: Matching compile runner format.
    *   `diagnostic`: A single diagnostic model representing the target compiler error.
*   **Outputs**:
    *   `suggestions`: Array of suggestions, replacement strings, explanation descriptions, and line replacement ranges.

---

### `POST /format`
Runs formatting tools on the current document.
*   **Inputs**:
    *   `code`: Text code to format.
    *   `language`: Compiler formatter target.
    *   `project_path`: Project folder containing formatting configurations.
*   **Outputs**:
    *   `formatted_code`: The reformatted code output text.
    *   `error`: Standard errors if the formatting tool fails.

---

### `POST /delete_file`
Deletes a target file or folder from the workspace.
*   **Inputs**:
    *   `path`: Location of the file/directory to delete.
    *   `is_dir`: Flag denoting if target is a folder.
*   **Outputs**: HTTP 200 OK status on success.

---

### `POST /copy_file`
Copies a source file or directory to a destination.
*   **Inputs**:
    *   `src_path`: Absolute path of source.
    *   `dest_path`: Destination path.
    *   `is_dir`: Flag denoting if target is a directory.
*   **Outputs**: HTTP 200 OK status on success.

---

### `POST /move_file`
Renames or moves a file or directory.
*   **Inputs**:
    *   `src_path`: Target file or directory to move.
    *   `dest_path`: Destination target path.
*   **Outputs**: HTTP 200 OK status on success.

---

### `POST /create_dir`
Creates a directory and any missing parent folders.
*   **Inputs**:
    *   `path`: Directory path to construct.
*   **Outputs**: HTTP 200 OK status on success.

---

### `POST /read_file`
Reads the content of a target file.
*   **Inputs**:
    *   `path`: Target file location.
*   **Outputs**:
    *   `content`: Full content of the read file.
    *   `error`: File system read errors.

---

### `POST /scan_project`
Recursively scan directory structure for navigation tree, automatically ignoring heavy directories (`node_modules`, `target`, `.git`, `.gradle`, etc.).
*   **Inputs**:
    *   `project_path`: Location of the target project.
*   **Outputs**:
    *   `files`: Array of `FileInfo` structures containing `rel_path` and `is_dir`.
    *   `error`: Scan execution errors.

---

### `POST /pick_directory`
Launches a native folder selection dialog based on the operating system.
*   **Outputs**:
    *   `success`: Boolean success flag.
    *   `path`: Chosen absolute directory path.
    *   `error`: Selection cancellations or platform failures.

---

### `POST /create_project`
Bootstraps folders and files for a supported language/framework workspace template.
*   **Inputs**:
    *   `name`: Name of the folder and package.
    *   `language`: Programming language identifier.
    *   `framework`: Scaffold framework (e.g. `vanilla`, `react`, `nextjs`, `vue`, `svelte`, `remix`).
    *   `path`: Absolute location.
*   **Outputs**:
    *   `success`: Boolean success flag.
    *   `error`: Bootstrap failures.

---

### `GET /file`
Serves a static file directly from local storage with the corresponding MIME type.
*   **Query Parameters**:
    *   `project_path`: Root folder of the project.
    *   `rel_path`: Target file path relative to project.
*   **Outputs**: The raw bytes of the file with matching Content-Type headers (images, videos, audio, PDF, etc.).

---

### 🖥️ Terminal API Router (`/terminal/*`)
Manages PTY terminal session lifecycle.

*   `POST /terminal/start`: Initiates a command shell session (`sh` or `cmd.exe`).
    *   **Inputs**: `project_path`
    *   **Outputs**: `session_id`
*   `POST /terminal/output`: Reads buffered terminal stdout/stderr stream.
    *   **Inputs**: `session_id`
    *   **Outputs**: `output`, `is_alive`
*   `POST /terminal/input`: Writes raw text data to the terminal's stdin stream.
    *   **Inputs**: `session_id`, `input`
    *   **Outputs**: `success`, `error`
*   `POST /terminal/stop`: Terminate the active shell session process.
    *   **Inputs**: `session_id`
    *   **Outputs**: `success`

---

### 🐙 Git API Router (`/git/*`)
Directly integrates standard git CLI operations.

*   `POST /git/status`: Retrieves branch status and tracked/untracked/modified status of all workspace files.
    *   **Inputs**: `project_path`
    *   **Outputs**: `branch`, `files` (array of paths and status), `error`
*   `POST /git/stage`: Adds file changes to staging area.
    *   **Inputs**: `project_path`, `file_path`
    *   **Outputs**: `success`, `output`, `error`
*   `POST /git/unstage`: Resets staged file changes.
    *   **Inputs**: `project_path`, `file_path`
    *   **Outputs**: `success`, `output`, `error`
*   `POST /git/discard`: Discards modifications or deletes untracked files.
    *   **Inputs**: `project_path`, `file_path`
    *   **Outputs**: `success`, `output`, `error`
*   `POST /git/commit`: Commits staged changes.
    *   **Inputs**: `project_path`, `message`
    *   **Outputs**: `success`, `output`, `error`
*   `POST /git/push`: Pushes committed changes to remote repository.
    *   **Inputs**: `project_path`
    *   **Outputs**: `success`, `output`, `error`
*   `POST /git/pull`: Pulls committed changes from remote repository.
    *   **Inputs**: `project_path`
    *   **Outputs**: `success`, `output`, `error`
*   `POST /git/diff_lines`: Retrieves granular line additions, edits, and deletions for syntax styling.
    *   **Inputs**: `project_path`, `file_path`
    *   **Outputs**: `added` (line numbers), `modified` (line numbers), `deleted` (line numbers)
*   `POST /git/diff_text`: Returns raw console representation of git diff output.
    *   **Inputs**: `project_path`, `file_path`
    *   **Outputs**: `success`, `output`, `error`
*   `POST /git/clone`: Clones a remote repository to local workspaces.
    *   **Inputs**: `clone_url`, `project_name`, `project_path`
    *   **Outputs**: `success`, `output`, `error`
*   `POST /git/log`: Retrieves history logs (commits details).
    *   **Inputs**: `project_path`
    *   **Outputs**: `commits` (list of commit hashes, subject, author, relative dates)
*   `POST /git/ai_commit_message`: Inspects staged changes and compiles recommendations of commit messages.
    *   **Inputs**: `project_path`
    *   **Outputs**: `message`, `suggestions` (list of commit strings), `error`

---

### 🔄 Live Server API Router (`/live-server/*`)
Manages hot-reloading development preview environments.

*   `POST /live-server/start`: Registers file system watcher and starts a local web server (port >= 5500) that auto-injects polling reload scripts into served HTML pages.
    *   **Inputs**: `project_path`
    *   **Outputs**: `port`
*   `POST /live-server/stop`: Shuts down the live development server.
    *   **Outputs**: `success` (bool)
*   `GET /live-server/status`: Inspects if a live server instance is active.
    *   **Outputs**: `running` (bool), `port`, `project_path`

---

### `GET /ping`
Checks backend status and responsiveness.
*   **Outputs**: Returns plain string confirming active server state.

## 🛠️ Termux Android Installation Details

Termux serves as the native runtime engine on Android. For detailed steps, see **[TERMUX_SETUP.md](./TERMUX_SETUP.md)**.

1.  **F-Droid Repository**:
    Do not download Termux from the Google Play Store (outdated packages and security warnings). Use F-Droid or direct APK download options.
2.  **Base Setup**:
    Initialize Termux packages:
    ```bash
    pkg update && pkg upgrade -y
    ```
3.  **Core Dependencies**:
    ```bash
    pkg install git rust clang build-essential nodejs-lts python go -y
    ```
4.  **Storage Integration**:
    Connect storage paths to ensure files are visible inside download directories:
    ```bash
    termux-setup-storage
    ```
    This grants Termux filesystem read/write permissions to internal shared storage maps.

---

## 📡 Cross-Device & iOS Network Connectivity

CodeDroid allows you to use your iPad or iPhone as a code editor screen, while using an Android tablet or local PC running Termux/Rust as the compiler server.

```
 ┌──────────────────────────┐         ┌────────────────────────────────┐
 │  iPhone / iPad / Browser │──WiFi──▶│  PC or Android (Termux Host)   │
 │                          │         │                                │
 │  Open in Safari/Chrome:  │         │  codedroid_api  → port 3000    │
 │  http://<HOST-IP>:8082   │         │  trunk serve    → port 8082    │
 └──────────────────────────┘         └────────────────────────────────┘
```

For detailed configurations, see **[NETWORK_ACCESS.md](./docs/NETWORK_ACCESS.md)**.

### Step-by-Step Device Bridging
1.  **Start Services with Open Bindings**:
    On your hosting PC or Android tablet, boot the API:
    ```bash
    cd codedroid_api && cargo run --release
    ```
    In another session, start the Trunk Web IDE specifying bindings:
    ```bash
    cd codedroid_frontend && trunk serve --port 8082 --address 0.0.0.0
    ```
2.  **Locate Local IP Address**:
    Find the hosting device's local routing IP on the network:
    *   *Android*: Run `ip addr show wlan0` (looks like `192.168.0.101`).
    *   *macOS*: Run `ipconfig getifaddr en0`.
    *   *Linux*: Run `hostname -I`.
3.  **Connect Remote Client**:
    Open Safari or Chrome on your secondary iPad/iPhone and go to `http://192.168.0.101:8082`.
4.  **Configure API Routing**:
    Open settings (⚙️) inside the Web IDE and set the **Backend Server** path to `http://192.168.0.101:3000`. Tap **Test** to establish connection.

---

## ⚙️ LSP Binary Executable Path Resolution

The backend implements custom lookup logic inside `utils.rs` (`resolve_lsp_executable`) to resolve compiler LSPs.

```
                  Start LSP Request
                          │
            Does executable exist in PATH?
                 (using which / where)
                 ├── Yes ──► Return binary command name
                 └── No
                          │
         Check Termux System Prefix ($PREFIX/bin/)
                 ├── Yes ──► Return path to Termux binary
                 └── No
                          │
       Check macOS Homebrew Binaries (/opt/homebrew/bin/)
                 ├── Yes ──► Return Homebrew path
                 └── No
                          │
        Check NPM Global Installations (~/.npm-global/bin/)
                 ├── Yes ──► Return global NPM binary
                 └── No
                          │
              Run default name fallback
```

---

## 🔮 Upcoming Features & Roadmap

We are expanding CodeDroid into a full-featured desktop-class editor on mobile:
*   **Origin Private File System (OPFS)**: Integrate the File System Access API to edit local folders on your phone directly from the browser.
*   **Collaborative Sessions**: Support multi-client peer-to-peer pairing over WebRTC for pair programming.
*   **Offline Native Compiler Toolchains**: Bundle minimal compiler binaries inside wrapper apps to run code completely detached from an external API server.

---

## 🤝 Contributing

We welcome contributions to CodeDroid. Please check **[CONTRIBUTING.md](./CONTRIBUTING.md)** for details on making pull requests, code formatting (`cargo fmt`), and setting up your dev workspace.

---

## 📄 License

CodeDroid is licensed under the [GNU General Public License v3.0](LICENSE).

---

## 💬 Community Channels
*   **Discord**: [Join our Community Server](https://discord.gg/5srCEqsht)
*   **Telegram**: [Join Channel Updates](https://t.me/codedroid133)
*   **YouTube**: [Watch Video Guides & Features](https://www.youtube.com/@CodeDroidYT)

---
*CodeDroid — Because real developers code everywhere.*