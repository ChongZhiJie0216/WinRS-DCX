# WinRS-DCX (DuinoDCX Management)

## Project Overview
WinRS-DCX is a Rust-based native Windows application designed to manage the Behringer Ultradrive Pro / DCX2496 audio processor. It communicates with the DCX2496 hardware over an RS232-to-USB serial connection. 

This project is an evolution of the original ESP32-based DuinoDCX controller. It replaces the Wi-Fi microcontroller requirement with a standalone desktop application. The application is built using Rust for the backend API and serial communication (`tokio`, `axum`, `serialport`), and embeds the existing web-based user interface (`dcx-ui`) directly into the final executable using the `rust-embed` crate. This allows the entire application to be distributed as a single `.exe` file without external UI dependencies.

## Architecture & Key Technologies
*   **Backend:** Rust (2021 Edition)
    *   `tokio`: Asynchronous runtime.
    *   `axum`: Web framework providing the local API (`/api/*`) for the UI to consume.
    *   `serialport`: For RS232 communication with the DCX2496 hardware.
    *   `rust-embed`: Embeds static web assets into the compiled binary.
*   **Frontend:** JavaScript/TypeScript (managed via npm as `dcx-ui`)
*   **Executable Configuration:** `winres` is used during the build process to attach the `Serial.ico` icon to the Windows executable.

## Building and Running

### Prerequisites
*   Windows OS
*   Rust Toolchain (`cargo`)
*   Node.js (`npm` - to fetch UI assets)

### Setup
1.  Run `npm install` in the root directory to download the `dcx-ui` package into `node_modules`. This provides the necessary static files for `rust-embed`.

### Building the Executable
To compile the project and generate the standalone binary:
1.  Execute `scripts\build.cmd`.
2.  This script cleans the project, runs `cargo build --release` inside the `duinodcx-rs` directory, refreshes the Windows icon cache, and outputs `WinRS-DCX-Management.exe` in the project root.

### Running in Development
To run the application with live logging without generating a standalone executable:
1.  Execute `scripts\start.bat`.
2.  This starts the local web server. The UI can be accessed at `http://127.0.0.1:3000`.

## Development Conventions
*   **Rust Code:** Follows standard `cargo fmt` and `cargo clippy` conventions. The Rust codebase is located in the `duinodcx-rs` directory.
*   **UI Updates:** If updates to the UI are required, the `dcx-ui` dependency in `package.json` must be updated, `npm install` executed, and the project rebuilt.
*   **Auto-Reconnect:** The backend features auto-reconnect logic. It continuously monitors the active COM port and attempts reconnection if the hardware is unplugged and plugged back in.
*   **Commit Messages:** Commit messages should be clear, concise, and ideally focused on the "why" of the change rather than just the "what".
