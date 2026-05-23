# 🦀 Rust Mobile Setup & Auto-suggestions Guide

Get a full, desktop-grade Rust development environment on your phone using CodeDroid and Termux.

---

## 🛠️ Step 1: Install Rust Toolchain & Compiler
Open Termux and run the following command to install the official Rust compiler (`rustc`) and package manager (`cargo`):

```bash
pkg install rust
```

Verify the installation:
```bash
rustc --version
cargo --version
```

---

## 🧠 Step 2: Install Language Server (LSP)
For real-time code completions, diagnostics, and hover hints, install `rust-analyzer`:

```bash
pkg install rust-analyzer
```

Verify that `rust-analyzer` is in your PATH:
```bash
rust-analyzer --version
```

---

## ✨ Step 3: Enable Auto-suggestions
CodeDroid API automatically detects `rust-analyzer` if it is installed in your Termux environment. 

1. Start your CodeDroid API server.
2. Open CodeDroid Web IDE and create/open any `.rs` file.
3. Start typing (e.g., `let`, `fn`, `std::`) and you will see completions instantly!
