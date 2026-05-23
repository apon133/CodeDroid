# 🪆 Kotlin Mobile Setup & Auto-suggestions Guide

Get a full Kotlin development environment on your phone using CodeDroid and Termux.

---

## 🛠️ Step 1: Install Kotlin Compiler
Open Termux and run the following command to install the Kotlin compiler:

```bash
pkg install kotlin
```

Verify the installation:
```bash
kotlinc -version
```

---

## 🧠 Step 2: Install Language Server (LSP)
For real-time code completions and formatting, install the `kotlin-language-server`:

```bash
pkg install kotlin-language-server
```

Verify that it's in your PATH:
```bash
kotlin-language-server --version
```

---

## ✨ Step 3: Enable Auto-suggestions
CodeDroid API automatically detects `kotlin-language-server` if it is installed in your Termux environment.

1. Start your CodeDroid API server.
2. Open CodeDroid Web IDE and create/open any `.kt` file.
3. Start typing and you will see completions instantly!
