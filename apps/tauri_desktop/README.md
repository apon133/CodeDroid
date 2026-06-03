# CodeDroid — Tauri Desktop Client

This directory contains the **Tauri-based native desktop wrapper** for CodeDroid. It packages the reactive Leptos Rust WebAssembly frontend into lightweight, native desktop clients for macOS, Windows, and Linux.

---

## 🚀 Why a Tauri Wrapper?

Tauri allows us to build cross-platform desktop applications with a secure, highly optimized native system WebView. Unlike Electron, Tauri apps:
- Have an extremely small footprint (usually under 15-20MB).
- Use significantly less RAM by sharing the operating system's native WebView runtime.
- Are written in Rust, aligning perfectly with the core CodeDroid backend and frontend crates.

---

## ✨ Features & Configurations

- **Native Desktop Integration**: Custom system window chrome, native performance, and hardware acceleration support.
- **Persistent Cache & Settings**: The OS-native WebView automatically saves configurations, settings, and file paths.
- **Offline Capability**: Embeds compiled WebAssembly assets directly into the native build target so the client loads instantly without remote network dependencies.

---

## 🛠️ Prerequisites

To run or build the Tauri desktop app, you need:
1. **Node.js** (LTS version recommended)
2. **pnpm** (preferred package manager)
3. **Rust Toolchain** (rustc, cargo)
4. **System WebView Dependencies**:
   - *macOS*: Installed by default.
   - *Windows*: WebView2 Runtime.
   - *Linux*: Webkit2GTK (e.g. `libwebkit2gtk-4.0-dev` / `libwebkit2gtk-4.1-dev` depending on distro).

---

## 🏁 Getting Started

### 1. Synchronize WebAssembly Assets
Compile the frontend and transfer the build output directly to the Tauri source directory:
```bash
# From the root directory of the CodeDroid workspace
./apps/sync_assets.sh
```

### 2. Install Dependencies
Run from the Tauri directory to install npm packages:
```bash
cd apps/tauri_desktop
pnpm install
```

### 3. Run Development Server
Launches the desktop window in debug mode with hot-reloading:
```bash
pnpm tauri dev
```

### 4. Build Installers
Generate optimized standalone installers (DMG/App for macOS, MSI/EXE for Windows, DEB/AppImage for Linux):
```bash
pnpm tauri build
```
The output bundles will be located in:
`src-tauri/target/release/bundle/`

---

## 📂 Project Structure Details

- `src/`: Compiled frontend assets synced from `codedroid_frontend/dist/`.
- `src-tauri/tauri.conf.json`: Main Tauri configurations including window size, styling, bundle identifiers, and security permissions.
- `src-tauri/src/main.rs`: Launches the Tauri window loop and handles native system menus or rust commands.
- `package.json` & `pnpm-lock.yaml`: Dependency declarations for Tauri CLI tools.
