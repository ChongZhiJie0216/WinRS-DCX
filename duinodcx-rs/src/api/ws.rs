// duinodcx-rs/src/api/ws.rs
use axum::{
    extract::{State, ws::{WebSocketUpgrade, WebSocket, Message}},
    response::IntoResponse,
};
use std::sync::Arc;
use crate::AppState;

pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

pub async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
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
