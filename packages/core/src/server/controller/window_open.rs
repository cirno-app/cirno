use crate::{AppError, AppState, server::ApiJson, server::ServiceClaim};
use axum::{debug_handler, extract::State};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use anyhow::Context;
use tao::{event_loop::EventLoop, window::WindowBuilder};
use wry::WebViewBuilder;

#[derive(Deserialize)]
pub struct Request {
    title: String,
    url: String,
}

#[derive(Serialize)]
pub struct Response {
    id: u8,
}

#[debug_handler]
pub async fn controller_window_open(
    State(app_state): State<Arc<AppState>>,
    claim: ServiceClaim,
) -> anyhow::Result<ApiJson<Response>, AppError> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .build(&event_loop)
        .context("Failed to create window")?;

    let builder = WebViewBuilder::new().with_url("https://tauri.app");

    #[cfg(not(target_os = "linux"))]
    let webview = builder.build(&window).context("Failed to create webview")?;
    #[cfg(target_os = "linux")]
    let webview = builder
        .build_gtk(window.gtk_window())
        .context("Failed to create webview")?;

    Ok(ApiJson(Response {}))
}