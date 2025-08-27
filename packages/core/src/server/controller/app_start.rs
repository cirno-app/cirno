use crate::server::ApiJson;
use crate::{AppError, AppState, server::AppClaim};
use axum::{
    debug_handler,
    extract::{Path, State},
};
use serde::Serialize;
use std::sync::Arc;

#[derive(Serialize)]
pub struct Response {}

#[debug_handler]
pub async fn controller_app_start(
    State(app_state): State<Arc<AppState>>,
    claim: AppClaim,
    Path(id): Path<String>,
) -> anyhow::Result<ApiJson<Response>, AppError> {
    Ok(ApiJson(Response {}))
}