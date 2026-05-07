# CodeDroid Web IDE — The Mobile Frontend

This is the official Web IDE for **CodeDroid**, built with **Rust** and the **Leptos** web framework. It compiles to **WebAssembly (WASM)** to provide a desktop-class, high-performance editor experience on mobile browsers.

## Features

- **Modern Editor**: High-performance text editing with syntax highlighting (Syntect).
- **IntelliSense**: Floating code suggestions powered by the CodeDroid API.
- **Dynamic File Explorer**: Manage your project files directly from the browser.
- **Integrated Terminal**: View stdout and stderr in real-time.
- **Web Preview**: Automatic detection of dev servers (Vite, React, etc.) with a built-in preview browser.
- **Responsive Design**: Tailored for mobile screens but fully functional on desktop.

## Local Development

### Prerequisites
- Rust (latest stable)
- `wasm32-unknown-unknown` target:
  ```bash
  rustup target add wasm32-unknown-unknown
  ```
- **Trunk**: The WASM web application bundler.
  ```bash
  cargo install --locked trunk
  ```

### Running Locally
```bash
trunk serve
```
Open `http://127.0.0.1:8080` in your browser. Ensure your **CodeDroid API** is running on port `3000`.

## Configuration

The IDE connects to the API server at `localhost:3000` by default. This can be configured in the application settings within the IDE.

## Deployment

The frontend is automatically deployed to **[codedroid.netlify.app](https://codedroid.netlify.app)**.

## License

GNU General Public License v3.0.
