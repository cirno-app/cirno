use crate::{
    AppError, AppState,
    server::{ApiJson, ServiceClaim},
};
use axum::{Json, debug_handler, extract::State, http::StatusCode};
use serde::Serialize;
use serde_json::Value;
use std::sync::Arc;

#[derive(Serialize)]
pub struct Response {}

#[debug_handler]
pub async fn controller_gc(
    State(app_state): State<Arc<AppState>>,
    claim: ServiceClaim,
) -> anyhow::Result<ApiJson<Response>, AppError> {
    Ok(ApiJson(Response {}))
}
