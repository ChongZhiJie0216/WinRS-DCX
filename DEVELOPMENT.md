# Development Setup Guide

This guide will help you set up the development environment for DuinoDCX Management.

## Prerequisites

To build and run this project, you need the following tools installed:

1.  **Rust**: Install the Rust toolchain from [rustup.rs](https://rustup.rs/).
    *   This is required to compile the `duinodcx-rs` backend.
2.  **Node.js**: Install Node.js from [nodejs.org](https://nodejs.org/).
    *   Required to manage the `dcx-ui` dependency and static assets.
3.  **Git**: To clone the repository.

## Project Structure

*   `duinodcx-rs/`: The Rust backend source code.
    *   `src/main.rs`: Web server, API routes, and serial management.
    *   `src/ultradrive.rs`: Protocol implementation for the DCX2496.
*   `node_modules/dcx-ui/`: The pre-built UI assets (installed via npm).
*   `build.cmd`: Windows script to build the standalone executable.
*   `start.bat`: Script to run the project using `cargo run`.

## Setting Up the Environment

1.  **Clone the repository**:
    ```bash
    git clone https://github.com/lasselukkari/DuinoDCX.git
    cd DuinoDCX
    ```

2.  **Install Node.js dependencies**:
    The UI assets are managed as an npm package.
    ```bash
    npm install
    ```
    *This will populate the `node_modules/dcx-ui/dist` folder which is required for embedding the UI.*

3.  **Build the Standalone Executable**:
    On Windows, you can use the provided build script:
    ```cmd
    build.cmd
    ```
    This will compile the Rust project in release mode and copy `duinodcx-management.exe` to the root directory.

## Running in Development Mode

To run the project with live logs and without creating a standalone executable:

1.  Open a terminal in the root directory.
2.  Run the start script:
    ```cmd
    start.bat
    ```
    *Note: This script runs `cargo run --release` inside the `duinodcx-rs` folder.*

## Key Features

### Standalone Binary
The project uses `rust-embed` to include the `dcx-ui` static files directly into the compiled executable. This means `duinodcx-management.exe` can be distributed as a single file without needing `node_modules`.

### Auto-Reconnect
The backend includes logic to monitor the serial connection. If the USB connection to the DCX2496 is lost, it will automatically attempt to reconnect to the last used COM port every 2 seconds.

### API Endpoint Reference
*   `GET /api/ports`: Lists available COM ports.
*   `PUT /api/port`: Selects a specific COM port.
*   `GET /api/state`: Returns the current state of the DCX device.
*   `POST /api/commands`: Sends direct protocol commands to the device.
