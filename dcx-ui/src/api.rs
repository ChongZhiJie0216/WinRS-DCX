use reqwasm::http::Request;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct ConnectionResponse {
    pub current: String,
    pub ip: String,
}

#[derive(Serialize)]
pub struct PortUpdate {
    pub port_name: String,
}

#[allow(dead_code)]
#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct VersionResponse {
    pub version: String,
    #[serde(rename = "buildDate")]
    pub build_date: String,
}

pub async fn fetch_ports() -> Result<Vec<String>, String> {
    Request::get("/api/ports")
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<Vec<String>>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn fetch_connection() -> Result<ConnectionResponse, String> {
    Request::get("/api/connection")
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<ConnectionResponse>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn connect_port(port_name: &str) -> Result<(), String> {
    let payload = PortUpdate { port_name: port_name.to_string() };
    let resp = Request::put("/api/port")
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&payload).unwrap())
        .send()
        .await
        .map_err(|e| e.to_string())?;
    
    if resp.ok() {
        Ok(())
    } else {
        Err(format!("Error: {}", resp.status()))
    }
}

pub async fn disconnect_port() -> Result<(), String> {
    let resp = Request::delete("/api/connection")
        .send()
        .await
        .map_err(|e| e.to_string())?;
    
    if resp.ok() {
        Ok(())
    } else {
        Err(format!("Error: {}", resp.status()))
    }
}
