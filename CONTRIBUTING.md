# 🤝 Contributing to CodeDroid

> Thank you for considering a contribution to CodeDroid! Every bug report, feature idea, and line of code helps make this a better tool for the mobile coding community.

---

## 🌈 Ways to Contribute

### 🐛 Reporting Bugs

Found something broken? [Open an issue](https://github.com/apon133/CodeDroid/issues) and include:

- A clear, descriptive title
- Steps to reproduce the bug
- Your device model and Android version
- Any relevant logs from Termux or the browser console

The more detail you give, the faster it gets fixed.

---

### 💡 Suggesting Features

Got an idea — like support for a new language or a UI improvement? Open an issue and describe:

- What the feature is
- Why it would be useful for mobile developers
- How you think it should work

---

### 🧑‍💻 Code Contributions

New to open source? No worries — here's the full flow:

1. **Fork the repository** on GitHub
2. **Create a feature branch:**
   ```bash
   git checkout -b feature/my-cool-feature
   ```
3. **Implement your changes:**
   - Backend changes → work inside `codedroid_api`
   - Frontend changes → work inside `codedroid_frontend`
4. **Test your changes** locally in Termux
5. **Commit with a clear message** using the [Conventional Commits](https://www.conventionalcommits.org/) format:
   ```bash
   git commit -m 'feat: add dark mode support'
   ```
6. **Push to your fork** and **open a Pull Request**

---

## 🛠️ Local Development Setup

### Backend (API Server)

| Property | Value |
|---|---|
| Language | Rust |
| Framework | Axum |
| Directory | `codedroid_api` |

```bash
cd codedroid_api
cargo run
```

### Frontend (Web IDE)

| Property | Value |
|---|---|
| Language | Rust (compiled to WASM) |
| Framework | Leptos |
| Build Tool | `trunk` |
| Directory | `codedroid_frontend` |

```bash
cd codedroid_frontend
trunk serve
```

> 💡 Make sure `trunk` is installed: `cargo install trunk`

---

## 📜 Coding Guidelines

- Follow [standard Rust formatting](https://github.com/rust-lang/rustfmt) — run `cargo fmt` before committing
- Document all new public functions and structs
- Keep the **mobile experience** in mind: performance and screen space are limited

---

## ⚖️ License

By contributing to CodeDroid, you agree that your contributions will be licensed under the [GNU General Public License v3.0](LICENSE).

---

Happy coding! 🚀