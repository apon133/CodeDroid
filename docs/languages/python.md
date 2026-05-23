# 🐍 Python Mobile Setup & Auto-suggestions Guide

Get a full Python development environment on your phone using CodeDroid and Termux.

---

## 🛠️ Step 1: Install Python Runtime
Open Termux and run the following command to install Python and pip:

```bash
pkg install python
```

Verify the installation:
```bash
python --version
pip --version
```

---

## 🧠 Step 2: Install Language Server (LSP)
For real-time code completions, formatting, and linting, install `python-lsp-server` (`pylsp`):

```bash
pip install python-lsp-server
```

Verify the installation:
```bash
pylsp --version
```

---

## ✨ Step 3: Enable Auto-suggestions
CodeDroid API automatically detects `pylsp` in your Termux environment to parse Python source code.

1. Start your CodeDroid API server.
2. Open CodeDroid Web IDE and create/open any `.py` file.
3. Start typing (e.g., `import`, `def`, `print`) and completions will be shown in real time!
