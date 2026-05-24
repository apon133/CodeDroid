# 📚 CodeDroid Docs

Welcome to the CodeDroid documentation! Below you'll find setup guides for every supported language and framework — each with Termux installation steps and LSP auto-suggestion configuration.

---

## 🌍 Web Technologies

| Language / Framework | File Extension | LSP Server | Guide |
|---|---|---|---|
| HTML | `.html` | `vscode-html-language-server` | [📄 html.md](languages/html.md) |
| CSS | `.css` | `vscode-css-language-server` | [📄 css.md](languages/css.md) |
| JavaScript | `.js` | `typescript-language-server` | [📄 javascript.md](languages/javascript.md) |
| TypeScript | `.ts` | `typescript-language-server` | [📄 typescript.md](languages/typescript.md) |
| JSX (React) | `.jsx` | `typescript-language-server` | [📄 jsx.md](languages/jsx.md) |
| TSX (React + TS) | `.tsx` | `typescript-language-server` | [📄 tsx.md](languages/tsx.md) |
| Vue | `.vue` | `vue-language-server` | [📄 vue.md](languages/vue.md) |
| Svelte | `.svelte` | `svelteserver` | [📄 svelte.md](languages/svelte.md) |

---

## 🖥️ Systems & General Purpose

| Language | File Extension | LSP Server | Guide |
|---|---|---|---|
| Rust | `.rs` | `rust-analyzer` | [📄 rust.md](languages/rust.md) |
| Go | `.go` | `gopls` | [📄 go.md](languages/go.md) |
| C | `.c` | `clangd` | [📄 c.md](languages/c.md) |
| C++ | `.cpp` | `clangd` | [📄 cpp.md](languages/cpp.md) |
| Python | `.py` | `pylsp` | [📄 python.md](languages/python.md) |

---

## 📱 Mobile & Platform Languages

| Language | File Extension | LSP Server | Guide |
|---|---|---|---|
| Dart / Flutter | `.dart` | `dart language-server` | [📄 dart.md](languages/dart.md) |
| Swift | `.swift` | `sourcekit-lsp` | [📄 swift.md](languages/swift.md) |
| Kotlin | `.kt` | `kotlin-language-server` | [📄 kotlin.md](languages/kotlin.md) |
| Java | `.java` | `jdtls` | [📄 java.md](languages/java.md) |

---

## 🔬 Other Languages

| Language | File Extension | LSP Server | Guide |
|---|---|---|---|
| Ruby | `.rb` | `solargraph` | [📄 ruby.md](languages/ruby.md) |
| Scala | `.scala` | *(coming soon)* | [📄 scala.md](languages/scala.md) |
| Haskell | `.hs` | *(coming soon)* | [📄 haskell.md](languages/haskell.md) |
| R | `.r` | *(coming soon)* | [📄 r.md](languages/r.md) |
| C# | `.cs` | *(coming soon)* | [📄 csharp.md](languages/csharp.md) |
| Perl | `.pl` | *(coming soon)* | [📄 perl.md](languages/perl.md) |
| Pascal | `.pas` | *(coming soon)* | [📄 pascal.md](languages/pascal.md) |

---

## 🚀 Quick Start (All Languages)

Every guide follows the same simple 3–5 step process:

1. **Install Node.js or runtime** via `pkg install`
2. **Install the LSP server** via `npm install -g` or language-specific package manager
3. **Create or open a project** in CodeDroid Web IDE
4. **Start typing** — completions appear automatically!

> 💡 **Tip:** CodeDroid API server must be running on `http://localhost:3000` for completions to work.
