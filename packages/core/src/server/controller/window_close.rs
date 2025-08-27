use crate::{
    AppError, AppState,
    server::{ApiJson, AppClaim},
};
use axum::{
    Json, debug_handler,
    extract::{Path, State},
    http::StatusCode,
};
use serde::Serialize;
use serde_json::Value;
use std::sync::Arc;

#[derive(Serialize)]
pub struct Response {}

#[debug_handler]
pub async fn controller_window_close(
    State(app_state): State<Arc<AppState>>,
    claim: AppClaim,
    Path(id): Path<String>,
) -> anyhow::Result<ApiJson<Response>, AppError> {
    Ok(ApiJson(Response {}))
}