# Backend Routing Refactor Design

## Purpose
Refactor the Rust backend (`duinodcx-rs`) to extract routing and handler logic out of `main.rs`. The goal is to improve maintainability, separate concerns (API vs. UI serving), and keep `main.rs` focused on server initialization and state management. The functionality of the application must remain exactly the same.

## Architecture

We will introduce a new `api` module directory within `duinodcx-rs/src/`.

### 1. `src/api/mod.rs`
- **Responsibility**: Wires up all the routes and exports a single `setup_router(state: Arc<AppState>) -> Router` function.
- **Dependencies**: Imports handlers from `routes`, `ui`, and `ws` submodules.

### 2. `src/api/routes.rs`
- **Responsibility**: Handles all standard JSON REST API endpoints.
- **Contents**:
  - `get_version`, `update_firmware`, `get_state`, `get_status`, `select_device`
  - `create_direct_command`, `list_ports`, `update_port`, `get_connection`
  - `update_connection`, `delete_connection`, `get_settings`, `update_settings`
  - All related data structures (`PortUpdate`, `ConnectionUpdate`, `ConnectionResponse`, `VersionResponse`, `SettingsResponse`, `SettingsUpdate`).

### 3. `src/api/ws.rs`
- **Responsibility**: Handles WebSocket lifecycle and binary message proxying.
- **Contents**:
  - `ws_handler`: The HTTP upgrade endpoint.
  - `handle_socket`: The async message loop for broadcasting device state to UI and forwarding UI commands to the device.

### 4. `src/api/ui.rs`
- **Responsibility**: Serves the embedded static assets and injects necessary UI scripts.
- **Contents**:
  - `struct Asset`: The `rust-embed` declaration for `../dcx-ui-old/dist/`.
  - `static_handler`: The fallback handler for serving JS/CSS/Assets.
  - `index_handler`: Reads `index.html` and injects the inline JavaScript needed to modify the React UI at runtime (relabeling inputs, adding baud rate drop-down, etc.).

### 5. `src/main.rs` Update
- **Responsibility**: Application entry point.
- **Changes**:
  - Remove all handler functions, payload structs, and the `Asset` struct.
  - Keep `AppState` definition.
  - Keep logging initialization, `Ultradrive` device manager initialization, and `ConnectionManager` setup.
  - Keep the background processing loop for incoming serial data.
  - Use `api::setup_router(shared_state)` to configure the axum `Router`.
  - Start the `tokio::net::TcpListener`.

## Data Flow
The data flow remains unchanged. `main.rs` creates the shared `AppState`, which contains thread-safe references (`Arc<Mutex<T>>`) to the hardware connections. The `Router` constructed by `api::mod::setup_router` is provided this state, which Axum distributes to the handlers in `routes.rs`, `ui.rs`, and `ws.rs`.

## Testing
Since this is a structural refactoring without behavioral changes, we will verify the change by compiling the project (`cargo check` / `cargo build`) and manually verifying that the UI loads and API endpoints respond correctly. No logic changes to the hardware interface or injected scripts will be made.
