# 🐦 Swift Mobile Setup & Auto-suggestions Guide

Get a full Swift development environment on your phone using CodeDroid and Termux.

---

## 🛠️ Step 1: Install Swift Compiler & Tools
Open Termux and run the following command to install the Swift compiler:

```bash
pkg install swift
```

Verify the installation:
```bash
swift --version
```

---

## 🧠 Step 2: Install Language Server (LSP)
For real-time code completions, Swift uses `sourcekit-lsp`. It is built into the toolchain or can be installed via:

```bash
pkg install sourcekit-lsp
```

Verify that it's in your PATH:
```bash
sourcekit-lsp --version
```

---

## ✨ Step 3: Enable Auto-suggestions
CodeDroid API automatically detects `sourcekit-lsp` if it is installed in your Termux environment.

1. Start your CodeDroid API server.
2. Open CodeDroid Web IDE and create/open any `.swift` file.
3. Start typing (e.g., `import`, `print`) and suggestions will pop up!
