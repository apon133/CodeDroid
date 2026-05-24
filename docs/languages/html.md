# 🌐 HTML Mobile Setup & Auto-suggestions Guide

Get a full HTML development environment on your phone using CodeDroid and Termux.

---

## 🛠️ Step 1: Install Node.js Runtime
Open Termux and run the following command to install the LTS version of Node.js:

```bash
pkg install nodejs-lts
```

Verify the installation:
```bash
node --version
npm --version
```

---

## 🧠 Step 2: Install Language Server (LSP)
For HTML auto-completions, tags suggestions, and diagnostics, we use `vscode-html-language-server` (provided by VSCode's extracted language servers):

```bash
npm install -g vscode-langservers-extracted
```

Verify the installation:
```bash
vscode-html-language-server --version
```

---

## ✨ Step 3: Enable Auto-suggestions
CodeDroid API automatically detects `vscode-html-language-server` if it is installed globally in npm.

1. Start your CodeDroid API server.
2. Open CodeDroid Web IDE and create/open any `.html` file.
3. Start typing (e.g., `div`, `html`, or elements inside `<body>`) and enjoy instant completion suggestions!
