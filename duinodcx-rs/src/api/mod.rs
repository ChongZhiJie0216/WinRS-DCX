// duinodcx-rs/src/api/mod.rs
pub mod ui;
pub mod ws;
pub mod routes;

use axum::{
    routing::{delete, get, patch, post, put},
    Router,
};
use std::sync::Arc;
use crate::AppState;

pub fn setup_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/ws", get(ws::ws_handler))
        .route("/api/version", get(routes::get_version))
        .route("/api/update", post(routes::update_firmware))
        .route("/api/state", get(routes::get_state))
        .route("/api/status", get(routes::get_status))
        .route("/api/selected", put(routes::select_device))
        .route("/api/commands", post(routes::create_direct_command))
        .route("/api/ports", get(routes::list_ports))
        .route("/api/port", put(routes::update_port))
        .route("/api/networks", get(routes::list_ports))
        .route("/api/connection", get(routes::get_connection))
        .route("/api/connection", patch(routes::update_connection))
        .route("/api/connection", delete(routes::delete_connection))
        .route("/api/settings", get(routes::get_settings))
        .route("/api/settings", patch(routes::update_settings))
        .fallback(ui::static_handler)
        .with_state(state)
}