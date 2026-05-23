# 🔌 C# Mobile Setup & Auto-suggestions Guide

Get a full C# development environment on your phone using CodeDroid and Termux.

---

## 🛠️ Step 1: Install .NET SDK
Open Termux and run the following command to install the dotnet environment:

```bash
pkg install dotnet-sdk
```

Verify the installation:
```bash
dotnet --version
```

---

## 🧠 Step 2: Enable Language Support (OmniSharp/Dotnet)
For auto-suggestions, CodeDroid integrates with the native dotnet suggestions or Omnisharp. Installing `dotnet-sdk` is enough to allow parsing of C# projects.

---

## ✨ Step 3: Enable Auto-suggestions
1. Start your CodeDroid API server.
2. Open CodeDroid Web IDE and create/open any `.cs` file.
3. Start typing (e.g., `Console.WriteLine`) and run your code on the fly!
