# 🐹 Go Mobile Setup & Auto-suggestions Guide

Get a full Go development environment on your phone using CodeDroid and Termux.

---

## 🛠️ Step 1: Install Go Compiler & Runtime
Open Termux and run the following command to install the Go programming language environment:

```bash
pkg install golang
```

Verify the installation:
```bash
go version
```

---

## 🧠 Step 2: Install Language Server (LSP)
For real-time code completions, diagnostics, and signature help, install the official Go language server (`gopls`):

```bash
pkg install gopls
```

Or install it via `go install` if the package is not found in your repository:
```bash
go install golang.org/x/tools/gopls@latest
```

Verify the installation:
```bash
gopls version
```

---

## ✨ Step 3: Enable Auto-suggestions
CodeDroid API automatically detects `gopls` if it is installed in your Termux environment.

1. Start your CodeDroid API server.
2. Open CodeDroid Web IDE and create/open any `.go` file.
3. Start typing (e.g., `func`, `import`, `fmt.Println`) and suggestions will pop up!
