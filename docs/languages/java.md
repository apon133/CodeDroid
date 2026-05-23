# ☕ Java Mobile Setup & Auto-suggestions Guide

Get a full Java development environment on your phone using CodeDroid and Termux.

---

## 🛠️ Step 1: Install OpenJDK
Open Termux and run the following command to install the Java compiler (`javac`) and runtime (`java`):

```bash
pkg install openjdk-17
```

Verify the installation:
```bash
java --version
javac --version
```

---

## 🧠 Step 2: Install Language Server (LSP)
For real-time code completions, diagnostics, and diagnostics, install the Eclipse Eclipse Java Language Server (`jdtls`):

```bash
pkg install jdtls
```

Verify the installation:
```bash
jdtls --version
```

---

## ✨ Step 3: Enable Auto-suggestions
CodeDroid API automatically detects `jdtls` in your Termux environment to parse Java class files.

1. Start your CodeDroid API server.
2. Open CodeDroid Web IDE and create/open any `.java` file.
3. Start typing (e.g., `System.out.println`) and enjoy instant completion suggestions!
