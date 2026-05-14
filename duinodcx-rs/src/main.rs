use anyhow::Result;
use axum::{
    extract::{Form, State, ws::{WebSocketUpgrade, WebSocket, Message}},
    http::header,
    response::IntoResponse,
    routing::{delete, get, patch, post, put},
    Router,
};
use std::sync::Arc;
use tokio::sync::{Mutex, broadcast};
use std::time::Duration;
use tokio::time::sleep;

mod ultradrive;
mod locations;
mod connection_manager;

use crate::ultradrive::Ultradrive;
use crate::connection_manager::{ConnectionManager, ConnectionCommand};

use serde::{Deserialize, Serialize};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../dcx-ui-old/dist/"]
struct Asset;

struct AppState {
    device_manager: Arc<Mutex<Ultradrive>>,
    connection_manager: ConnectionManager,
    current_port: Arc<Mutex<String>>,
    ws_tx: broadcast::Sender<Vec<u8>>,
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

#[derive(Serialize)]
struct VersionResponse {
    version: String,
    #[serde(rename = "buildDate")]
    build_date: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    let (ws_tx, _) = broadcast::channel(100);
    
    let mut dm = Ultradrive::new(None);
    dm.set_ws_tx(ws_tx.clone());
    let device_manager = Arc::new(Mutex::new(dm));
    
    let connection_manager = ConnectionManager::new(device_manager.clone(), Duration::from_secs(30));
    
    let shared_state = Arc::new(AppState {
        device_manager: device_manager.clone(),
        connection_manager,
        current_port: Arc::new(Mutex::new(String::new())),
        ws_tx: ws_tx.clone(),
    });

    // Background Processing Task for incoming data
    let dm_proc = device_manager.clone();
    tokio::spawn(async move {
        loop {
            let mut dm = dm_proc.lock().await;
            if let Err(e) = dm.process_incoming() {
                log::error!("Error processing incoming: {:?}", e);
                dm.close_port();
            }
            drop(dm);
            sleep(Duration::from_millis(10)).await;
        }
    });

    let app = Router::new()
        .route("/api/ws", get(ws_handler))
        .route("/api/version", get(get_version))
        .route("/api/update", post(update_firmware))
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

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    log::info!("Server running at http://localhost:3000.");
    axum::serve(listener, app).await?;

    Ok(())
}

async fn get_version() -> impl IntoResponse {
    axum::Json(VersionResponse {
        version: "1.0.0".to_string(),
        build_date: "2026-05-14".to_string(),
    })
}

async fn update_firmware() -> impl IntoResponse {
    (axum::http::StatusCode::OK, "Firmware update received (Simulated)")
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    let mut rx = state.ws_tx.subscribe();
    
    loop {
        tokio::select! {
            Ok(msg) = rx.recv() => {
                if socket.send(Message::Binary(msg.into())).await.is_err() {
                    break;
                }
            }
            Some(msg) = socket.recv() => {
                match msg {
                    Ok(Message::Binary(data)) => {
                        let mut dm = state.device_manager.lock().await;
                        if let Err(e) = dm.process_outgoing(&data) {
                            log::error!("Error sending WS data to device: {}", e);
                        }
                    }
                    Ok(Message::Close(_)) | Err(_) => {
                        break;
                    }
                    _ => {}
                }
            }
        }
    }
}

async fn static_handler(uri: axum::http::Uri) -> axum::response::Response {
    let path = uri.path().trim_start_matches('/');
    if path.is_empty() || path == "index.html" { return index_handler().await; }
    match Asset::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
        }
        None => {
            if path.contains('.') { return (axum::http::StatusCode::NOT_FOUND, "404 Not Found").into_response(); }
            index_handler().await
        }
    }
}

async fn index_handler() -> axum::response::Response {
    match Asset::get("index.html") {
        Some(content) => {
            let html_str = String::from_utf8_lossy(&content.data);
            let injected_script = r#"
<script>
document.addEventListener('DOMContentLoaded', () => {
    const observer = new MutationObserver(() => {
        document.querySelectorAll('label').forEach(label => {
            if (label.innerText === 'Network' || label.innerText === 'Device / COM Port') {
                if (!label.dataset.refreshInjected) {
                    label.innerText = 'Device / COM Port';
                    label.dataset.refreshInjected = "true";
                    label.style.display = 'flex';
                    label.style.justifyContent = 'space-between';
                    label.style.alignItems = 'center';
                    const refreshBtn = document.createElement('button');
                    refreshBtn.innerHTML = '&#x21bb;'; // refresh icon
                    refreshBtn.style.border = 'none';
                    refreshBtn.style.background = 'none';
                    refreshBtn.style.color = '#007bff';
                    refreshBtn.style.cursor = 'pointer';
                    refreshBtn.onclick = (e) => {
                        e.preventDefault();
                        window.location.reload();
                    };
                    label.appendChild(refreshBtn);
                }
            }
            if (label.innerText === 'Password') {
                label.innerText = 'Baud Rate';
                const input = label.nextElementSibling;
                if (input && input.tagName === 'INPUT' && input.type === 'password') {
                    if (!input.dataset.baudInjected) {
                        input.dataset.baudInjected = "true";
                        input.style.display = 'none';
                        const select = document.createElement('select');
                        select.className = input.className;
                        ['9600','19200','38400','57600','115200'].forEach(b => {
                            const opt = document.createElement('option');
                            opt.value = b; opt.innerText = b;
                            if (b === '38400') opt.selected = true;
                            select.appendChild(opt);
                        });
                        const setNativeValue = (element, value) => {
                            const valueSetter = Object.getOwnPropertyDescriptor(element, 'value').set;
                            const prototype = Object.getPrototypeOf(element);
                            const prototypeValueSetter = Object.getOwnPropertyDescriptor(prototype, 'value').set;
                            if (valueSetter && valueSetter !== prototypeValueSetter) {
                                prototypeValueSetter.call(element, value);
                            } else {
                                valueSetter.call(element, value);
                            }
                        };
                        select.addEventListener('change', (e) => {
                            setNativeValue(input, e.target.value);
                            input.dispatchEvent(new Event('input', { bubbles: true }));
                        });
                        input.parentNode.insertBefore(select, input.nextSibling);
                        setNativeValue(input, '38400');
                        input.dispatchEvent(new Event('input', { bubbles: true }));
                    }
                }
            }
            if (label.innerText === 'IP') label.innerText = 'Connection Status';
        });

        document.querySelectorAll('button').forEach(btn => {
            if (btn.innerText === 'Connect') {
                btn.style.backgroundColor = '#dc3545';
                btn.style.borderColor = '#dc3545';
            } else if (btn.innerText === 'Disconnect') {
                btn.style.backgroundColor = '#28a745';
                btn.style.borderColor = '#28a745';
            }
        });
        
        document.querySelectorAll('input[disabled]').forEach(input => {
            if (input.value === 'Disconnected') {
                input.style.backgroundColor = '#dc3545';
                input.style.color = 'white';
            } else if (input.value.startsWith('Serial')) {
                input.style.backgroundColor = '#28a745';
                input.style.color = 'white';
            }
        });
    });
    observer.observe(document.body, { childList: true, subtree: true });
});
</script>
</body>
"#;
            let new_html = html_str.replace("</body>", injected_script);
            ([(header::CONTENT_TYPE, "text/html")], new_html).into_response()
        },
        None => (axum::http::StatusCode::NOT_FOUND, "404 Not Found").into_response(),
    }
}

async fn list_ports() -> impl IntoResponse {
    match serialport::available_ports() {
        Ok(ports) => {
            let mut port_names: Vec<String> = ports.into_iter().map(|p| p.port_name).collect();
            if port_names.is_empty() {
                #[cfg(target_os = "windows")]
                {
                    port_names = vec!["COM1".into(), "COM2".into(), "COM3".into(), "COM4".into()];
                }
                #[cfg(not(target_os = "windows"))]
                {
                    port_names = vec![
                        "/dev/ttyUSB0".into(), "/dev/ttyUSB1".into(),
                        "/dev/ttyS0".into(), "/dev/ttyS1".into(),
                        "/dev/ttyAMA0".into(), "/dev/ttyACM0".into()
                    ];
                }
            }
            axum::Json(port_names).into_response()
        }
        Err(_) => {
            #[cfg(target_os = "windows")]
            let fallback = vec!["COM1".into(), "COM2".into(), "COM3".into(), "COM4".into()];
            #[cfg(not(target_os = "windows"))]
            let fallback = vec![
                "/dev/ttyUSB0".into(), "/dev/ttyUSB1".into(),
                "/dev/ttyS0".into(), "/dev/ttyS1".into(),
                "/dev/ttyAMA0".into(), "/dev/ttyACM0".into()
            ];
            axum::Json(fallback).into_response()
        }
    }
}

async fn update_port(State(state): State<Arc<AppState>>, axum::Json(payload): axum::Json<PortUpdate>) -> impl IntoResponse {
    let _ = state.connection_manager.send_command(ConnectionCommand::Connect { 
        port: payload.port_name.clone(), 
        baud_rate: 38400 
    }).await;
    let mut current = state.current_port.lock().await;
    *current = payload.port_name;
    (axum::http::StatusCode::OK, "Port updated")
}

async fn get_connection(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let current = state.current_port.lock().await;
    axum::Json(ConnectionResponse { current: current.clone(), ip: "Serial".into() })
}

async fn delete_connection(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let _ = state.connection_manager.send_command(ConnectionCommand::Disconnect).await;
    let mut current = state.current_port.lock().await;
    *current = String::new();
    axum::Json(ConnectionResponse { current: String::new(), ip: "Disconnected".into() })
}

async fn update_connection(State(state): State<Arc<AppState>>, Form(payload): Form<ConnectionUpdate>) -> impl IntoResponse {
    let baud_rate = payload.password.as_deref().and_then(|s| s.parse::<u32>().ok()).unwrap_or(38400);
    let _ = state.connection_manager.send_command(ConnectionCommand::Connect { port: payload.ssid.clone(), baud_rate }).await;
    let mut current_port = state.current_port.lock().await;
    *current_port = payload.ssid.clone();
    axum::Json(ConnectionResponse { current: payload.ssid, ip: format!("Serial ({})", baud_rate) })
}

async fn get_state(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let mut buf = Vec::new();
    let dm = state.device_manager.lock().await;
    buf.push(dm.get_selected() as u8);
    dm.write_device(&mut buf);
    dm.write_devices(&mut buf);
    ([(header::CONTENT_TYPE, "application/binary")], buf)
}

async fn get_status(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let mut buf = Vec::new();
    let dm = state.device_manager.lock().await;
    dm.write_device_status(&mut buf);
    ([(header::CONTENT_TYPE, "application/binary")], buf)
}

async fn select_device(State(state): State<Arc<AppState>>, body: String) -> impl IntoResponse {
    if let Ok(id) = body.trim().parse::<usize>() {
        let mut dm = state.device_manager.lock().await;
        dm.set_selected(id);
        (axum::http::StatusCode::NO_CONTENT, "")
    } else { (axum::http::StatusCode::BAD_REQUEST, "Invalid device ID") }
}

async fn create_direct_command(State(state): State<Arc<AppState>>, body: axum::body::Bytes) -> impl IntoResponse {
    let mut dm = state.device_manager.lock().await;
    if let Err(_) = dm.process_outgoing(&body) { return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Failed").into_response(); }
    (axum::http::StatusCode::NO_CONTENT, "").into_response()
}

#[derive(Serialize)] struct SettingsResponse { ap_ssid: String, ap_password: String, auth: String, mdns_host: String, flow_control: String, auto_disable_ap: String }
#[derive(Deserialize)] struct SettingsUpdate { ap_ssid: String, ap_password: String }

async fn get_settings(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let current_port = state.current_port.lock().await;
    axum::Json(SettingsResponse { ap_ssid: current_port.clone(), ap_password: "38400".into(), auth: "Basic YWRtaW46YWRtaW4=".into(), mdns_host: "ultradrive".into(), flow_control: "0".into(), auto_disable_ap: "0".into() })
}

async fn update_settings(State(state): State<Arc<AppState>>, Form(payload): Form<SettingsUpdate>) -> impl IntoResponse {
    let baud_rate = payload.ap_password.parse::<u32>().unwrap_or(38400);
    let _ = state.connection_manager.send_command(ConnectionCommand::Connect { port: payload.ap_ssid.clone(), baud_rate }).await;
    let mut current_port = state.current_port.lock().await;
    *current_port = payload.ap_ssid.clone();
    (axum::http::StatusCode::OK, "Settings updated")
}
