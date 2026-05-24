# 💚 Vue.js Mobile Setup & Auto-suggestions Guide

Get a full Vue.js development environment on your phone using CodeDroid and Termux.

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

Vue uses the official `@vue/language-server` (also called `vue-language-server`):

```bash
npm install -g @vue/language-server typescript
```

Verify:
```bash
vue-language-server --version
```

---

## 📦 Step 3: Create a Vue Project

Inside Termux, scaffold a new Vue 3 project using Vite:

```bash
cd /sdcard/Download/codedroid
npm create vite@latest my-vue-app -- --template vue
cd my-vue-app
npm install
```

For Vue + TypeScript:
```bash
npm create vite@latest my-vue-ts-app -- --template vue-ts
cd my-vue-ts-app
npm install
```

---

## ⚙️ Step 4: Configure the LSP (Required for Vue)

Create a `jsconfig.json` (or `tsconfig.json`) at your project root:

```json
{
  "compilerOptions": {
    "target": "ESNext",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "jsx": "preserve"
  }
}
```

Also make sure `vite.config.js` exists (Vite template includes this by default).

---

## ✨ Step 5: Enable Auto-suggestions

1. Start your CodeDroid API server.
2. Open CodeDroid Web IDE and open any `.vue` file inside the project.
3. Start typing inside `<template>`, `<script>`, or `<style>` sections and get intelligent Vue 3 completions!

---

## 📝 Sample `.vue` File

```vue
<template>
  <div class="app">
    <h1>{{ title }}</h1>
  </div>
</template>

<script setup>
import { ref } from 'vue'

const title = ref('Hello Vue!')
</script>

<style scoped>
.app {
  font-family: sans-serif;
}
</style>
```
