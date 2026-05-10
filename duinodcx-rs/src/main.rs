use anyhow::Result;
use axum::{
    extract::{Form, State},
    http::header,
    response::IntoResponse,
    routing::{delete, get, patch, post, put},
    Router,
};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time::sleep;

mod ultradrive;
mod locations;

use crate::ultradrive::Ultradrive;

use serde::{Deserialize, Serialize};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../node_modules/dcx-ui/dist/"]
struct Asset;

struct AppState {
    device_manager: Mutex<Ultradrive>,
    current_port: Mutex<String>,
}

#[derive(Deserialize)]
struct PortUpdate {
    port_name: String,
}

#[derive(Deserialize)]
struct ConnectionUpdate {
    ssid: String,
    password: Option<String>,
}

#[derive(Serialize)]
struct ConnectionResponse {
    current: String,
    ip: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger with timestamps
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    let device_manager = Ultradrive::new(None);
    let shared_state = Arc::new(AppState {
        device_manager: Mutex::new(device_manager),
        current_port: Mutex::new(String::new()),
    });

    // Background Processing Task
    let dm_proc = shared_state.clone();
    tokio::spawn(async move {
        let mut last_reconnect_attempt = Instant::now();
        let reconnect_interval = Duration::from_secs(2);

        loop {
            let mut needs_reconnect = false;
            let mut target_port = String::new();

            {
                if let Ok(mut dm) = dm_proc.device_manager.lock() {
                    if let Err(e) = dm.process_incoming() {
                        log::error!("Error processing incoming: {:?}. Closing port for reconnect.", e);
                        dm.close_port();
                    }
                    
                    // Check if we have a target port but no active connection
                    if dm.get_port_active().is_none() {
                        if let Ok(current) = dm_proc.current_port.lock() {
                            if !current.is_empty() {
                                target_port = current.clone();
                                needs_reconnect = true;
                            }
                        }
                    }
                }
            }

            if needs_reconnect && last_reconnect_attempt.elapsed() >= reconnect_interval {
                last_reconnect_attempt = Instant::now();
                log::info!("Attempting to reconnect to {}...", target_port);
                
                let baud_rate = 38400;
                match serialport::new(&target_port, baud_rate)
                    .timeout(Duration::from_millis(10))
                    .open() {
                        Ok(port) => {
                            log::info!("Successfully reconnected to {}", target_port);
                            if let Ok(mut dm) = dm_proc.device_manager.lock() {
                                dm.set_port(port);
                            }
                        }
                        Err(e) => {
                            log::debug!("Reconnect attempt failed: {}", e);
                        }
                    }
            }

            sleep(Duration::from_millis(10)).await;
        }
    });

    // Axum Router
    let app = Router::new()
        .route("/api/state", get(get_state))
        .route("/api/status", get(get_status))
        .route("/api/selected", put(select_device))
        .route("/api/commands", post(create_direct_command))
        .route("/api/ports", get(list_ports))
        .route("/api/port", put(update_port))
        .route("/api/networks", get(list_ports))
        .route("/api/connection", get(get_connection))
        .route("/api/connection", patch(update_connection))
        .route("/api/connection", delete(delete_connection))
        .route("/api/settings", get(get_settings))
        .route("/api/settings", patch(update_settings))
        .fallback(static_handler)
        .with_state(shared_state);

    // Run Web Server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    log::info!("Server running at http://0.0.0.0:3000. No serial port connected by default.");
    axum::serve(listener, app).await?;

    Ok(())
}

async fn static_handler(uri: axum::http::Uri) -> axum::response::Response {
    let path = uri.path().trim_start_matches('/');

    if path.is_empty() || path == "index.html" {
        return index_handler().await;
    }

    match Asset::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
        }
        None => {
            if path.contains('.') {
                return (axum::http::StatusCode::NOT_FOUND, "404 Not Found").into_response();
            }
            index_handler().await
        }
    }
}

async fn index_handler() -> axum::response::Response {
    match Asset::get("index.html") {
        Some(content) => ([(header::CONTENT_TYPE, "text/html")], content.data).into_response(),
        None => (axum::http::StatusCode::NOT_FOUND, "404 Not Found").into_response(),
    }
}

async fn list_ports() -> impl IntoResponse {
    match serialport::available_ports() {
        Ok(ports) => {
            let mut port_names: Vec<String> = ports.into_iter().map(|p| p.port_name).collect();
            if port_names.is_empty() {
                // Fallback for Windows if no ports detected automatically
                port_names = vec![
                    "COM1".to_string(), "COM2".to_string(), "COM3".to_string(), "COM4".to_string(),
                    "COM5".to_string(), "COM6".to_string(), "COM7".to_string(), "COM8".to_string(),
                ];
            }
            log::info!("Available ports: {:?}", port_names);
            axum::Json(port_names).into_response()
        }
        Err(e) => {
            log::error!("Failed to list ports: {}", e);
            axum::Json(vec![
                "COM1".to_string(), "COM2".to_string(), "COM3".to_string(), "COM4".to_string(),
            ]).into_response()
        }
    }
}

async fn update_port(State(state): State<Arc<AppState>>, axum::Json(payload): axum::Json<PortUpdate>) -> impl IntoResponse {
    let baud_rate = 38400;
    
    {
        let mut dm = state.device_manager.lock().unwrap();
        dm.close_port();
    }

    match serialport::new(&payload.port_name, baud_rate)
        .timeout(Duration::from_millis(10))
        .open() {
            Ok(port) => {
                let mut dm = state.device_manager.lock().unwrap();
                dm.set_port(port);
                let mut current = state.current_port.lock().unwrap();
                *current = payload.port_name;
                (axum::http::StatusCode::OK, "Port updated").into_response()
            }
            Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to open port {}: {}", payload.port_name, e)).into_response(),
        }
}

async fn get_connection(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let current = state.current_port.lock().unwrap();
    axum::Json(ConnectionResponse {
        current: current.clone(),
        ip: "Serial".to_string(), // Use "Serial" instead of IP
    })
}

async fn delete_connection(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    {
        let mut dm = state.device_manager.lock().unwrap();
        dm.close_port();
    }
    let mut current = state.current_port.lock().unwrap();
    *current = String::new();
    axum::Json(ConnectionResponse {
        current: String::new(),
        ip: "Disconnected".to_string(),
    })
}

async fn update_connection(State(state): State<Arc<AppState>>, Form(payload): Form<ConnectionUpdate>) -> impl IntoResponse {
    let baud_rate = payload.password
        .as_deref()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(38400);

    // Explicitly close the existing port before opening the new one
    // to release the system lock, especially if it's the same port.
    {
        let mut dm = state.device_manager.lock().unwrap();
        dm.close_port();
    }

    match serialport::new(&payload.ssid, baud_rate)
        .timeout(Duration::from_millis(10))
        .open() {
            Ok(port) => {
                let mut dm = state.device_manager.lock().unwrap();
                dm.set_port(port);
                let mut current_port = state.current_port.lock().unwrap();
                *current_port = payload.ssid.clone();
                
                axum::Json(ConnectionResponse {
                    current: payload.ssid,
                    ip: format!("Serial ({})", baud_rate),
                }).into_response()
            }
            Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to open port {} at {}: {}", payload.ssid, baud_rate, e)).into_response(),
        }
}

async fn get_state(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let mut buf = Vec::new();
    {
        let dm = state.device_manager.lock().unwrap();
        buf.push(dm.get_selected() as u8);
        dm.write_device(&mut buf);
        dm.write_devices(&mut buf);
    }
    (
        [(header::CONTENT_TYPE, "application/binary")],
        buf,
    )
}

async fn get_status(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let mut buf = Vec::new();
    {
        let dm = state.device_manager.lock().unwrap();
        dm.write_device_status(&mut buf);
    }
    (
        [(header::CONTENT_TYPE, "application/binary")],
        buf,
    )
}

async fn select_device(State(state): State<Arc<AppState>>, body: String) -> impl IntoResponse {
    if let Ok(id) = body.trim().parse::<usize>() {
        let mut dm = state.device_manager.lock().unwrap();
        dm.set_selected(id);
        (axum::http::StatusCode::NO_CONTENT, "")
    } else {
        (axum::http::StatusCode::BAD_REQUEST, "Invalid device ID")
    }
}

async fn create_direct_command(State(state): State<Arc<AppState>>, body: axum::body::Bytes) -> impl IntoResponse {
    let mut dm = state.device_manager.lock().unwrap();
    if let Err(e) = dm.process_outgoing(&body) {
        log::error!("Error processing outgoing command: {:?}", e);
        return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Failed to send command");
    }
    (axum::http::StatusCode::NO_CONTENT, "")
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SettingsResponse {
    ap_ssid: String,
    ap_password: String,
    auth: String,
    mdns_host: String,
    flow_control: String,
    auto_disable_ap: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SettingsUpdate {
    ap_ssid: String,
    ap_password: String,
    #[allow(dead_code)]
    auth: String,
    #[allow(dead_code)]
    mdns_host: String,
    #[allow(dead_code)]
    flow_control: String,
    #[allow(dead_code)]
    auto_disable_ap: String,
}

async fn get_settings(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let current_port = state.current_port.lock().unwrap();
    axum::Json(SettingsResponse {
        ap_ssid: current_port.clone(),
        ap_password: "38400".to_string(),
        auth: "Basic YWRtaW46YWRtaW4=".to_string(), // admin:admin
        mdns_host: "ultradrive".to_string(),
        flow_control: "0".to_string(),
        auto_disable_ap: "0".to_string(),
    })
}

async fn update_settings(State(state): State<Arc<AppState>>, Form(payload): Form<SettingsUpdate>) -> impl IntoResponse {
    // Handle settings update by attempting to open the port
    let baud_rate = payload.ap_password.parse::<u32>().unwrap_or(38400);
    
    {
        let mut dm = state.device_manager.lock().unwrap();
        dm.close_port();
    }

    match serialport::new(&payload.ap_ssid, baud_rate)
        .timeout(Duration::from_millis(10))
        .open() {
            Ok(port) => {
                let mut dm = state.device_manager.lock().unwrap();
                dm.set_port(port);
                let mut current_port = state.current_port.lock().unwrap();
                *current_port = payload.ap_ssid.clone();
                (axum::http::StatusCode::OK, "Settings updated").into_response()
            }
            Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to open port {}: {}", payload.ap_ssid, e)).into_response(),
        }
}
