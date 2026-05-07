# CodeDroid API — The Mobile Execution Engine

This is the backend server for **CodeDroid**, a high-performance code execution engine and IDE for mobile devices. It is written in **Rust** and designed to run efficiently inside **Termux** on Android.

## Features

- **Multi-language Support**: Executes code in 13+ languages including Rust, Python, Go, C++, etc.
- **LSP Integration**: Provides real-time code completions via language servers (rust-analyzer, clangd, gopls, etc.).
- **Process Management**: Handles long-running processes (like dev servers) with PID-based control.
- **Package Management**: Automatically installs dependencies using `npm`, `pip`, `cargo`, and more.
- **File Syncing**: Synchronizes code changes from the IDE to the local file system.

## Endpoints

- `POST /run`: Executes code and returns stdout/stderr.
- `POST /complete`: Returns LSP-powered code suggestions.
- `POST /sync_file`: Syncs file content to the device storage.
- `POST /stop`: Safely terminates a running process by PID.
- `POST /add_package`: Installs a library for a specific language.

## Local Development

### Prerequisites
- Rust (latest stable)
- Compilers/Runtimes for the languages you want to test.

### Running the server
```bash
cargo run --release
```
The server defaults to port `3000`.

## Architecture

The API uses **Axum** for routing and **Tokio** for asynchronous process execution. Each request is handled by spawning a child process of the respective system compiler or runtime, ensuring "real" execution rather than sandboxed emulation.

## License

GNU General Public License v3.0.
