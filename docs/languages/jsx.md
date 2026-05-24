# ⚛️ JSX (React) Mobile Setup & Auto-suggestions Guide

Get a full React/JSX development environment on your phone using CodeDroid and Termux.

---

## 🛠️ Step 1: Install Node.js Runtime

Open Termux and run:

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

JSX uses `typescript-language-server` with `react-jsx` support:

```bash
npm install -g typescript-language-server typescript
```

Verify:
```bash
typescript-language-server --version
```

---

## 📦 Step 3: Create a React Project

Inside Termux, go to your CodeDroid project folder and scaffold a Vite-based React app:

```bash
cd /sdcard/Download/codedroid
npm create vite@latest my-react-app -- --template react
cd my-react-app
npm install
```

CodeDroid API will **automatically generate** a `jsconfig.json` at the project root the first time you open a `.jsx` file — no manual config needed.

---

## ✨ Step 4: Enable Auto-suggestions

1. Start your CodeDroid API server.
2. Open CodeDroid Web IDE and open any `.jsx` file inside the project.
3. Start typing (e.g., `<div`, `useState`, `useEffect`) and enjoy instant React-aware completion suggestions!

---

## 📝 Sample `jsconfig.json` (auto-generated)

If you want to manually create it, place this at your project root:

```json
{
  "compilerOptions": {
    "jsx": "react-jsx",
    "target": "ESNext",
    "module": "ESNext",
    "moduleResolution": "node"
  }
}
```
