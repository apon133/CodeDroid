# CodeDroid — Flutter Android Client

This directory contains the **Flutter-based native wrapper** for CodeDroid, specifically optimized for Android devices. It packages the reactive Leptos Rust WebAssembly frontend into a high-performance Android application shell.

---

## 🚀 Why a Flutter Wrapper?

Web browsers have strict security constraints regarding the `file://` protocol. Specifically:
1. **WebAssembly Loading**: Modern web browsers block loading and compiling WebAssembly binary streams from `file://` assets due to MIME type and CORS restrictions.
2. **Persistence**: Features like LocalStorage, session storage, and cookies require a valid origin (HTTP/HTTPS) to be persisted between launches.

To solve this, the Flutter client implements **`InAppLocalhostServer`**. It spins up a secure, local HTTP server (`http://localhost:8080`) directly on the device background thread to serve the frontend assets with correct MIME types (`application/wasm`, etc.) offline.

---

## ✨ Features & Configurations

- **Seamless Dark Theme**: Styled with a premium dark theme (`#181818` background, `#228DF2` blue accents) that matches the CodeDroid editor perfectly, eliminating safe-area/notch borders.
- **Native System UI Overlay**: Configured transparent status bars and matching navigation bars for a fully immersive, edge-to-edge coding workspace.
- **Automatic File Picker**: Handled natively using the system file browser wrapper so you can upload or select files in your project directories.
- **Persistent LocalStorage & Cache**: DOM storage, Web SQL database, and custom caching configurations are fully enabled.
- **LSP Local Host Routing**: Built-in network permission profiles allow connection to system compilers and LSP backend servers running on localhost ports (e.g. `http://localhost:3000`).

---

## 🛠️ Prerequisites

Before running or building the app, make sure you have the following installed:
1. **Flutter SDK** (v3.10 or higher recommended)
2. **Android SDK** (configured with command line tools and emulator/physical device debugging enabled)

---

## 🏁 Getting Started

### 1. Synchronize WebAssembly Assets
Ensure the latest build of `codedroid_frontend` is synced into the Flutter project's assets:
```bash
# From the root directory of the CodeDroid workspace
./apps/sync_assets.sh
```

### 2. Run the Application
Start the app on your connected device/emulator:
```bash
cd apps/flutter_android
flutter run
```

### 3. Build Release APK
Generate an optimized, standalone release bundle:
```bash
flutter build apk --release
```
The compiled APK will be located at:
`build/app/outputs/flutter-apk/app-release.apk`

---

## 📂 Project Structure Details

- `lib/main.dart`: Entry point. Launches the localhost server, configures the `ThemeData`, and embeds the custom `InAppWebView` container.
- `android/app/src/main/AndroidManifest.xml`: Configures standard internet, hardware acceleration, and cleartext traffic privileges to enable local backend API loopbacks.
- `pubspec.yaml`: Declares dependencies such as `flutter_inappwebview` and asset mapping paths.
