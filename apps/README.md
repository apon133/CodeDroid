# CodeDroid Apps Integration

This directory contains the cross-platform wrapper applications for CodeDroid, allowing you to run the Leptos Rust frontend on Android, macOS, Linux, and Windows.

## Directory Structure

- `flutter_android/`: Flutter application configured specifically for **Android**.
- `tauri_desktop/`: Tauri application configured for desktop platforms (**macOS, Windows, Linux**).
- `sync_assets.sh`: A shell script that automatically builds the latest frontend in release mode and copies the compiled assets to both wrapper applications.

---

## 🛠️ How to Sync Assets

Every time you change the frontend code (in `codedroid_frontend`), run the sync script to compile the new version and update the apps:

```bash
./apps/sync_assets.sh
```

---

## 📱 Flutter (Android) Wrapper

The Flutter wrapper is located in `flutter_android/`. It runs a secure local HTTP server (`http://localhost:8080`) to host the frontend. This is essential for:
- Correct WebAssembly loading (browsers restrict WebAssembly loads over the `file://` protocol due to strict MIME types and CORS rules).
- Persistence of LocalStorage, cookies, and app cache.
- Native performance and offline execution.

### Configurations Applied:
1. **Internet Permissions**: Enabled `android.permission.INTERNET` and enabled cleartext HTTP traffic in `AndroidManifest.xml` (so it can connect to `http://localhost:3000` backend server).
2. **File Picker**: Android file choosing (`onShowFileChooser`) is fully enabled and handled natively by the WebView.
3. **Storage & Cache**: `domStorageEnabled`, `databaseEnabled`, and Web Cache are enabled with persistent storage.
4. **Compile SDK**: Updated `minSdkVersion` to `21` for `flutter_inappwebview` compatibility.

### How to Run:
Ensure you have an Android device or emulator connected:
```bash
cd apps/flutter_android
flutter run
```

### How to Build APK:
```bash
cd apps/flutter_android
flutter build apk --release
```

---

## 💻 Tauri (Desktop) Wrapper

The Tauri wrapper is located in `tauri_desktop/` and uses `pnpm`.

### Configurations Applied:
1. **Internet Permissions**: Full internet and network access enabled natively.
2. **File Picker**: Native HTML file picker support out of the box.
3. **Cache Saving**: Natively handles persistent state via the OS web view.

### How to Run:
```bash
cd apps/tauri_desktop
pnpm tauri dev
```

### How to Build Installers (macOS DMG/App, Windows MSI, Linux DEB/AppImage):
```bash
cd apps/tauri_desktop
pnpm tauri build
```
