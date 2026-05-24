# 🟦 TSX (React + TypeScript) Mobile Setup & Auto-suggestions Guide

Get a full React + TypeScript development environment on your phone using CodeDroid and Termux.

---

## 🛠️ Step 1: Install Node.js Runtime

Open Termux and run:

```bash
pkg install nodejs-lts
```

Verify:
```bash
node --version
npm --version
```

---

## 🧠 Step 2: Install Language Server (LSP)

TSX uses `typescript-language-server` with full TypeScript + JSX support:

```bash
npm install -g typescript-language-server typescript
```

Verify:
```bash
typescript-language-server --version
```

---

## 📦 Step 3: Create a React + TypeScript Project

Inside Termux, scaffold a Vite-based React TS app:

```bash
cd /sdcard/Download/codedroid
npm create vite@latest my-react-ts-app -- --template react-ts
cd my-react-ts-app
npm install
```

CodeDroid API will **automatically generate** a `tsconfig.json` when you open a `.tsx` file.

---

## ✨ Step 4: Enable Auto-suggestions

1. Start your CodeDroid API server.
2. Open CodeDroid Web IDE and open any `.tsx` file inside the project.
3. Start typing (e.g., `<Button`, `React.FC`, `useState<string>`) and get full TypeScript-aware completions!

---

## 📝 Sample `tsconfig.json`

```json
{
  "compilerOptions": {
    "target": "ESNext",
    "module": "ESNext",
    "jsx": "react-jsx",
    "strict": true,
    "moduleResolution": "node",
    "esModuleInterop": true,
    "skipLibCheck": true
  },
  "include": ["src"]
}
```
