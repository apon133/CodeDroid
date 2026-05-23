# 👾 C++ Mobile Setup & Auto-suggestions Guide

Get a full C++ development environment on your phone using CodeDroid and Termux.

---

## 🛠️ Step 1: Install C++ Compiler & Tools
Open Termux and run the following command to install the C++ compiler (Clang/G++) and build utilities:

```bash
pkg install clang build-essential
```

Verify the installation:
```bash
clang++ --version
```

---

## 🧠 Step 2: Install Language Server (LSP)
For C++, the recommended LSP is `clangd`. It is already included in the `clang` package in Termux, but can be installed/updated via:

```bash
pkg install clangd
```

Verify the installation:
```bash
clangd --version
```

---

## ✨ Step 3: Enable Auto-suggestions
CodeDroid API automatically detects `clangd` in your Termux environment to parse C++ source files.

1. Start your CodeDroid API server.
2. Open CodeDroid Web IDE and create/open any `.cpp` or `.cc` file.
3. Start typing (e.g., `#include <iostream>`, `std::cout`) and suggestions will pop up!
