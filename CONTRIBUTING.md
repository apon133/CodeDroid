# Contributing to CodeDroid

First off, thank you for considering contributing to CodeDroid! It's people like you who make CodeDroid a great tool for the mobile coding community.

## 🌈 How Can I Contribute?

### Reporting Bugs
If you find a bug, please [open an issue](https://github.com/apon133/CodeDroid/issues). Be sure to include:
- A clear, descriptive title.
- Steps to reproduce the bug.
- Your device model and Android version.
- Any relevant logs from Termux or the browser console.

### Suggesting Features
We love new ideas! If you want to suggest a new feature (like support for a new language or a UI improvement), please open an issue and describe:
- What the feature is.
- Why it would be useful for mobile developers.
- How you think it should work.

### Code Contributions
1.  **Fork the repository**.
2.  **Create a feature branch**: `git checkout -b feature/my-cool-feature`.
3.  **Implement your changes**.
    - For API changes, work in `codedroid_api`.
    - For Frontend changes, work in `codedroid_frontend`.
4.  **Test your changes** locally in Termux.
5.  **Commit with a clear message**: `git commit -m 'feat: add dark mode support'`.
6.  **Push to your fork** and **open a Pull Request**.

## 🛠️ Local Development Setup

### API Server (Backend)
- Language: **Rust**
- Framework: **Axum**
- To run: `cd codedroid_api && cargo run`

### Web IDE (Frontend)
- Language: **Rust (WASM)**
- Framework: **Leptos**
- Tools required: `trunk`
- To run: `cd codedroid_frontend && trunk serve`

## 📜 Coding Guidelines
- Follow [Standard Rust Styling](https://github.com/rust-lang/rustfmt).
- Document new public functions and structures.
- Keep the mobile experience in mind: performance and screen space are key!

## ⚖️ License
By contributing to CodeDroid, you agree that your contributions will be licensed under the project's [GNU General Public License v3.0](LICENSE).

Happy coding! 🚀
