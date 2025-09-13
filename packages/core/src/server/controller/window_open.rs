use crate::{
    AppError, AppState,
    server::{ApiJson, ServiceClaim},
};
use anyhow::{Context, Error};
use axum::{debug_handler, extract::State};
use serde::Serialize;
use std::sync::Arc;
use tao::window::WindowBuilder;
use wry::WebViewBuilder;

#[derive(Serialize)]
pub struct Response {}

#[debug_handler]
pub async fn controller_window_open(
    State(app_state): State<Arc<AppState>>,
    claim: ServiceClaim,
) -> anyhow::Result<ApiJson<Response>, AppError> {
    let (id, state) = app_state.wry.create()?;

    Ok(ApiJson(Response {}))
}
