use anyhow::Result;
use axum::{
    extract::{Form, State},
    http::header,
    response::IntoResponse,
    routing::{delete, get, patch, post, put},
    Router,
};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::sleep;

mod ultradrive;
mod locations;

use crate::ultradrive::Ultradrive;

use serde::{Deserialize, Serialize};

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

    // Open Serial Port (Change COM port as needed)
    let port_name = "COM1"; // Default
    let baud_rate = 38400;

    let initial_port = match serialport::new(port_name, baud_rate)
        .timeout(Duration::from_millis(10))
        .open() {
            Ok(p) => Some(p),
            Err(e) => {
                log::warn!("Failed to open initial port {}: {}. Will start without active port.", port_name, e);
                None
            }
        };

    // Use a dummy port if initial fails, Ultradrive needs a port to start
    // or we can wrap the port in Option in Ultradrive. 
    // For now, let's just use a Null port if it fails, but Ultradrive expects Box<dyn SerialPort>.
    // Better: let's modify Ultradrive to take Option<Box<dyn SerialPort>> or provide a dummy.
    // Actually, serialport doesn't have a Null port. 
    // Let's use a more robust approach: keep Ultradrive as is, but handle the first open better.
    
    // If we can't open COM4, let's try to find ANY port.
    let port = if let Some(p) = initial_port {
        p
    } else {
        let ports = serialport::available_ports()?;
        if let Some(p_info) = ports.first() {
            log::info!("Trying alternative port: {}", p_info.port_name);
            serialport::new(&p_info.port_name, baud_rate)
                .timeout(Duration::from_millis(10))
                .open()?
        } else {
            // If no ports at all, we might need a dummy implementation of SerialPort trait 
            // or modify Ultradrive. Let's assume there's at least one port for now or it fails.
            // Actually, let's just bail if NO ports are found, but warn if COM1 specifically fails.
            anyhow::bail!("No serial ports found and default COM1 failed.");
        }
    };

    let port_name_str = port.name().unwrap_or_else(|| port_name.to_string());
    let current_active_port = port_name_str.clone();
    let device_manager = Ultradrive::new(port);
    let shared_state = Arc::new(AppState {
        device_manager: Mutex::new(device_manager),
        current_port: Mutex::new(port_name_str),
    });

    // Background Processing Task
    let dm_proc = shared_state.clone();
    tokio::spawn(async move {
        loop {
            {
                if let Ok(mut dm) = dm_proc.device_manager.lock() {
                    if let Err(e) = dm.process_incoming() {
                        log::error!("Error processing incoming: {:?}", e);
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
        .fallback_service(tower_http::services::ServeDir::new("../node_modules/dcx-ui/dist").fallback(tower_http::services::ServeFile::new("../node_modules/dcx-ui/dist/index.html")))
        .with_state(shared_state);

    // Run Web Server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    log::info!("Server running at http://0.0.0.0:3000 using port {}", current_active_port);
    axum::serve(listener, app).await?;

    Ok(())
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
