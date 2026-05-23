# 🟥 Scala Mobile Setup & Auto-suggestions Guide

Get a full Scala development environment on your phone using CodeDroid and Termux.

---

## 🛠️ Step 1: Install Scala Compiler & Tools
Open Termux and run the following command to install the Scala environment:

```bash
pkg install scala
```

Verify the installation:
```bash
scala -version
scalac -version
```

---

## 🧠 Step 2: Enable Language Support
CodeDroid compiles and executes Scala scripts using `scalac` and `scala` runtimes in Termux.

---

## ✨ Step 3: Enable Auto-suggestions
1. Start your CodeDroid API server.
2. Open CodeDroid Web IDE and create/open any `.scala` file.
3. Start typing (e.g., `object Main { ... }`) and run it immediately!
