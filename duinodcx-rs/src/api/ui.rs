// duinodcx-rs/src/api/ui.rs
use axum::{http::header, response::IntoResponse};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../dcx-ui/dist/"]
pub struct Asset;

pub async fn static_handler(uri: axum::http::Uri) -> axum::response::Response {
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

pub async fn index_handler() -> axum::response::Response {
    match Asset::get("index.html") {
        Some(content) => {
            ([(header::CONTENT_TYPE, "text/html")], content.data).into_response()
        },
        None => (axum::http::StatusCode::NOT_FOUND, "404 Not Found (Run dx build in dcx-ui first)").into_response(),
    }
}
