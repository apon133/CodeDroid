# 🧡 Svelte Mobile Setup & Auto-suggestions Guide

Get a full Svelte development environment on your phone using CodeDroid and Termux.

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

Svelte uses its own official language server:

```bash
npm install -g svelte-language-server typescript
```

Verify:
```bash
svelteserver --version
```

---

## 📦 Step 3: Create a Svelte Project

Inside Termux, scaffold a new Svelte project using Vite:

```bash
cd /sdcard/Download/codedroid
npm create vite@latest my-svelte-app -- --template svelte
cd my-svelte-app
npm install
```

For Svelte + TypeScript:
```bash
npm create vite@latest my-svelte-ts-app -- --template svelte-ts
cd my-svelte-ts-app
npm install
```

---

## ⚙️ Step 4: Configure the LSP (Recommended)

Create a `jsconfig.json` at your project root for better completions:

```json
{
  "compilerOptions": {
    "target": "ESNext",
    "module": "ESNext",
    "moduleResolution": "bundler"
  }
}
```

If using TypeScript, use `tsconfig.json` instead (Vite svelte-ts template includes this).

---

## ✨ Step 5: Enable Auto-suggestions

1. Start your CodeDroid API server.
2. Open CodeDroid Web IDE and open any `.svelte` file inside the project.
3. Start typing inside `<script>`, `<style>`, or markup sections and enjoy Svelte-native completions!

---

## 📝 Sample `.svelte` File

```svelte
<script>
  let count = 0;

  function increment() {
    count += 1;
  }
</script>

<main>
  <h1>Count: {count}</h1>
  <button on:click={increment}>Click me</button>
</main>

<style>
  main {
    font-family: sans-serif;
    text-align: center;
  }
</style>
```
