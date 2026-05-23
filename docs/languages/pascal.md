# 🌀 Pascal Mobile Setup & Auto-suggestions Guide

Get a full Pascal development environment on your phone using CodeDroid and Termux.

---

## 🛠️ Step 1: Install Free Pascal Compiler (FPC)
Open Termux and run the following command to install the Free Pascal Compiler:

```bash
pkg install fpc
```

Verify the installation:
```bash
fpc -i
```

---

## 🧠 Step 2: Enable Language Support
CodeDroid compiles and executes Pascal files using the `fpc` compiler inside your Termux environment.

---

## ✨ Step 3: Enable Auto-suggestions
1. Start your CodeDroid API server.
2. Open CodeDroid Web IDE and create/open any `.pas` file.
3. Start typing (e.g., `writeln('Hello World');`) and compile/run instantly!
