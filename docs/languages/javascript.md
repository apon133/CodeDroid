# 🌐 JavaScript Mobile Setup & Auto-suggestions Guide

Get a full JavaScript development environment on your phone using CodeDroid and Termux.

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
For JavaScript completions, we use the `typescript-language-server` which has excellent support for JS files via jsdoc/type inference:

```bash
npm install -g typescript-language-server typescript
```

Verify the installation:
```bash
typescript-language-server --version
```

---

## ✨ Step 3: Enable Auto-suggestions
CodeDroid API automatically detects `typescript-language-server` if it is installed globally in npm.

1. Start your CodeDroid API server.
2. Open CodeDroid Web IDE and create/open any `.js` file.
3. Start typing (e.g., `console.log`, `function`) and enjoy instant completion suggestions!
