use crate::{AppError, AppState, server::ApiJson};
use axum::{debug_handler, extract::State};
use serde::Serialize;
use std::sync::Arc;

#[derive(Serialize)]
pub struct Response {
    id: u8,
}

#[debug_handler]
pub async fn controller_window_open(
    State(app_state): State<Arc<AppState>>,
    claim: ServiceClaim,
) -> anyhow::Result<ApiJson<Response>, AppError> {
    let (id, state) = app_state.wry.create()?;

    Ok(ApiJson(Response { id }))
}
