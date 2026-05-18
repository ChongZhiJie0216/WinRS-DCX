// duinodcx-rs/src/main.rs
use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, Mutex};
use tokio::time::sleep;

mod api;
mod connection_manager;
mod locations;
mod ultradrive;

use crate::connection_manager::ConnectionManager;
use crate::ultradrive::Ultradrive;

pub struct AppState {
    pub device_manager: Arc<Mutex<Ultradrive>>,
    pub connection_manager: ConnectionManager,
    pub current_port: Arc<Mutex<String>>,
    pub ws_tx: broadcast::Sender<Vec<u8>>,
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

    let connection_manager =
        ConnectionManager::new(device_manager.clone(), Duration::from_secs(30));

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

    let app = api::setup_router(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    log::info!("Server running at http://localhost:3000.");
    axum::serve(listener, app).await?;

    Ok(())
}
