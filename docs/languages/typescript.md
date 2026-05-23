# 🟦 TypeScript Mobile Setup & Auto-suggestions Guide

Get a full TypeScript development environment on your phone using CodeDroid and Termux.

---

## 🛠️ Step 1: Install Node.js & Compiler
Open Termux and run the following command to install Node.js:

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
For TypeScript completions, diagnostics, and signature help, install the `typescript-language-server` globally:

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
2. Open CodeDroid Web IDE and create/open any `.ts` file.
3. Start typing (e.g., `let x: number = 5`) and enjoy instant completion suggestions!
