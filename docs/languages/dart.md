# 🎯 Dart Mobile Setup & Auto-suggestions Guide

Get a full Dart development environment on your phone using CodeDroid and Termux.

---

## 🛠️ Step 1: Install Dart SDK
Open Termux and run the following command to install the Dart SDK:

```bash
pkg install dart
```

Verify the installation:
```bash
dart --version
```

---

## 🧠 Step 2: Enable Language Server (LSP)
The Dart SDK comes with its language server pre-built. It is accessed via:

```bash
dart language-server
```

CodeDroid automatically resolves the path to the internal Dart language server when running Dart projects.

---

## ✨ Step 3: Enable Auto-suggestions
CodeDroid API automatically uses `dart language-server` when you open any `.dart` file.

1. Start your CodeDroid API server.
2. Open CodeDroid Web IDE and create/open any `.dart` file.
3. Start typing (e.g., `void main()`, `print`) and enjoy instant auto-completions!
