// duinodcx-rs/src/api/routes.rs
use crate::connection_manager::ConnectionCommand;
use crate::AppState;
use axum::{
    extract::{Form, State},
    http::header,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Deserialize)]
pub struct PortUpdate {
    pub port_name: String,
}

#[derive(Deserialize)]
pub struct ConnectionUpdate {
    pub ssid: String,
    pub password: Option<String>,
}

#[derive(Serialize)]
pub struct ConnectionResponse {
    pub current: String,
    pub ip: String,
}

#[derive(Serialize)]
pub struct VersionResponse {
    pub version: String,
    #[serde(rename = "buildDate")]
    pub build_date: String,
}

#[derive(Serialize)]
pub struct SettingsResponse {
    pub ap_ssid: String,
    pub ap_password: String,
    pub auth: String,
    pub mdns_host: String,
    pub flow_control: String,
    pub auto_disable_ap: String,
}

#[derive(Deserialize)]
pub struct SettingsUpdate {
    pub ap_ssid: String,
    pub ap_password: String,
}

pub async fn get_version() -> impl IntoResponse {
    axum::Json(VersionResponse {
        version: "1.0.0".to_string(),
        build_date: "2026-05-14".to_string(),
    })
}

pub async fn update_firmware() -> impl IntoResponse {
    (
        axum::http::StatusCode::OK,
        "Firmware update received (Simulated)",
    )
}

pub async fn list_ports() -> impl IntoResponse {
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
                        "/dev/ttyUSB0".into(),
                        "/dev/ttyUSB1".into(),
                        "/dev/ttyS0".into(),
                        "/dev/ttyS1".into(),
                        "/dev/ttyAMA0".into(),
                        "/dev/ttyACM0".into(),
                    ];
                }
            }
            axum::Json(port_names).into_response()
        }
        Err(_) => {
            #[cfg(target_os = "windows")]
            let fallback: Vec<String> =
                vec!["COM1".into(), "COM2".into(), "COM3".into(), "COM4".into()];
            #[cfg(not(target_os = "windows"))]
            let fallback: Vec<String> = vec![
                "/dev/ttyUSB0".into(),
                "/dev/ttyUSB1".into(),
                "/dev/ttyS0".into(),
                "/dev/ttyS1".into(),
                "/dev/ttyAMA0".into(),
                "/dev/ttyACM0".into(),
            ];
            axum::Json(fallback).into_response()
        }
    }
}

pub async fn update_port(
    State(state): State<Arc<AppState>>,
    axum::Json(payload): axum::Json<PortUpdate>,
) -> impl IntoResponse {
    let _ = state
        .connection_manager
        .send_command(ConnectionCommand::Connect {
            port: payload.port_name.clone(),
            baud_rate: 38400,
        })
        .await;
    let mut current = state.current_port.lock().await;
    *current = payload.port_name;
    (axum::http::StatusCode::OK, "Port updated")
}

pub async fn get_connection(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let current = state.current_port.lock().await;
    axum::Json(ConnectionResponse {
        current: current.clone(),
        ip: "Serial".into(),
    })
}

pub async fn delete_connection(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let _ = state
        .connection_manager
        .send_command(ConnectionCommand::Disconnect)
        .await;
    let mut current = state.current_port.lock().await;
    *current = String::new();
    axum::Json(ConnectionResponse {
        current: String::new(),
        ip: "Disconnected".into(),
    })
}

pub async fn update_connection(
    State(state): State<Arc<AppState>>,
    Form(payload): Form<ConnectionUpdate>,
) -> impl IntoResponse {
    let baud_rate = payload
        .password
        .as_deref()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(38400);
    let _ = state
        .connection_manager
        .send_command(ConnectionCommand::Connect {
            port: payload.ssid.clone(),
            baud_rate,
        })
        .await;
    let mut current_port = state.current_port.lock().await;
    *current_port = payload.ssid.clone();
    axum::Json(ConnectionResponse {
        current: payload.ssid,
        ip: format!("Serial ({})", baud_rate),
    })
}

pub async fn get_state(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let mut buf = Vec::new();
    let dm = state.device_manager.lock().await;
    buf.push(dm.get_selected() as u8);
    dm.write_device(&mut buf);
    dm.write_devices(&mut buf);
    ([(header::CONTENT_TYPE, "application/binary")], buf)
}

pub async fn get_status(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let mut buf = Vec::new();
    let dm = state.device_manager.lock().await;
    dm.write_device_status(&mut buf);
    ([(header::CONTENT_TYPE, "application/binary")], buf)
}

pub async fn select_device(State(state): State<Arc<AppState>>, body: String) -> impl IntoResponse {
    if let Ok(id) = body.trim().parse::<usize>() {
        let mut dm = state.device_manager.lock().await;
        dm.set_selected(id);
        (axum::http::StatusCode::NO_CONTENT, "")
    } else {
        (axum::http::StatusCode::BAD_REQUEST, "Invalid device ID")
    }
}

pub async fn create_direct_command(
    State(state): State<Arc<AppState>>,
    body: axum::body::Bytes,
) -> impl IntoResponse {
    let mut dm = state.device_manager.lock().await;
    if dm.process_outgoing(&body).is_err() {
        return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Failed").into_response();
    }
    (axum::http::StatusCode::NO_CONTENT, "").into_response()
}

pub async fn get_settings(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let current_port = state.current_port.lock().await;
    axum::Json(SettingsResponse {
        ap_ssid: current_port.clone(),
        ap_password: "38400".into(),
        auth: "Basic YWRtaW46YWRtaW4=".into(),
        mdns_host: "ultradrive".into(),
        flow_control: "0".into(),
        auto_disable_ap: "0".into(),
    })
}

pub async fn update_settings(
    State(state): State<Arc<AppState>>,
    Form(payload): Form<SettingsUpdate>,
) -> impl IntoResponse {
    let baud_rate = payload.ap_password.parse::<u32>().unwrap_or(38400);
    let _ = state
        .connection_manager
        .send_command(ConnectionCommand::Connect {
            port: payload.ap_ssid.clone(),
            baud_rate,
        })
        .await;
    let mut current_port = state.current_port.lock().await;
    *current_port = payload.ap_ssid.clone();
    (axum::http::StatusCode::OK, "Settings updated")
}
