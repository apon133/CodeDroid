# 💎 Ruby Mobile Setup & Auto-suggestions Guide

Get a full Ruby development environment on your phone using CodeDroid and Termux.

---

## 🛠️ Step 1: Install Ruby Runtime
Open Termux and run the following command to install Ruby and gems:

```bash
pkg install ruby
```

Verify the installation:
```bash
ruby --version
gem --version
```

---

## 🧠 Step 2: Install Language Server (LSP)
For real-time code completions, diagnostics, and documentation, install the `solargraph` gem:

```bash
gem install solargraph
```

Verify that it's in your PATH:
```bash
solargraph --version
```

---

## ✨ Step 3: Enable Auto-suggestions
CodeDroid API automatically detects `solargraph` if it is installed in your Termux environment.

1. Start your CodeDroid API server.
2. Open CodeDroid Web IDE and create/open any `.rb` file.
3. Start typing (e.g., `def`, `puts`) and suggestions will pop up!
