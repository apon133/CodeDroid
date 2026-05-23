# 🟪 Haskell Mobile Setup & Auto-suggestions Guide

Get a full Haskell development environment on your phone using CodeDroid and Termux.

---

## 🛠️ Step 1: Install Haskell Compiler (GHC)
Open Termux and run the following command to install the Glasgow Haskell Compiler (GHC) and runtime tools:

```bash
pkg install ghc
```

Verify the installation:
```bash
ghc --version
runhaskell --version
```

---

## 🧠 Step 2: Enable Language Support
CodeDroid compiles and runs Haskell code using `runhaskell` or `ghc` inside Termux.

---

## ✨ Step 3: Enable Auto-suggestions
1. Start your CodeDroid API server.
2. Open CodeDroid Web IDE and create/open any `.hs` file.
3. Start typing (e.g., `main = putStrLn "Hello"`) and run your code!
