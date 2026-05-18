# Dioxus UI Foundation Rebuild Design

## Purpose
Rebuild the foundation of the `WinRS-DCX` frontend using Rust and Dioxus, replacing the legacy React `dcx-ui-old`. This phase focuses solely on scaffolding the new workspace, configuring the build pipeline, establishing backend communication (REST + WebSocket), and implementing the initial connection management UI. 

## Architecture

### 1. Workspace Structure
- Create a new Rust crate `dcx-ui` at the root of the project.
- It will be a standard Rust application configured for Dioxus Web (`dioxus = { version = "0.5", features = ["web"] }`).
- A `Dioxus.toml` file will be created to configure the `dx` CLI builder.

### 2. State Management & Data Flow
- **Global State:** Dioxus signals (`use_signal`, `use_context_provider`) will be used to manage application-wide state:
  - `ConnectionState`: Tracks whether the device is connected, the active port, and the baud rate.
  - `DeviceState`: Holds the binary/parsed state from the Behringer Ultradrive.
- **REST Client:** `reqwasm::http::Request` will be used to interact with the backend API:
  - `GET /api/ports` to list available COM ports.
  - `PUT /api/port` to request a connection to a specific port.
  - `GET /api/connection` to retrieve current connection status.
- **WebSocket:** `gloo-net::websocket::futures::WebSocket` will connect to `ws://[host]/api/ws` to listen for real-time binary updates from the backend and send direct commands.

### 3. UI Components (Phase 1)
- **App Component:** The root component that initializes context providers and manages the layout.
- **Connection Panel:**
  - A dropdown list populated by `/api/ports`.
  - A dropdown for standard baud rates (e.g., 9600, 19200, 38400, 57600, 115200).
  - A "Connect"/"Disconnect" toggle button.
  - Visual indicators for connection status (e.g., green for connected, red for disconnected).

### 4. Build Pipeline & Integration
- **Backend Embedding:** Update `duinodcx-rs/src/api/ui.rs`'s `Asset` struct to target `#[folder = "../../dcx-ui/dist/"]`. The inline JavaScript injection currently in `ui.rs` (which manipulated the legacy React DOM) will be removed, as the new Dioxus UI will natively support the necessary inputs.
- **Build Scripts:** `scripts/build.cmd` will be updated to require `dx` (the Dioxus CLI) and run `dx build --release` inside the `dcx-ui` folder instead of running `npm run build` in `dcx-ui-old`.

## Future Phases (Out of Scope for this Spec)
- EQ, Crossovers, Routing, Delays, Limiters, and Plot rendering.
- Parsing the raw binary data from the WebSocket into strictly typed Rust structs within the UI (to mirror the old `parser.js`).
