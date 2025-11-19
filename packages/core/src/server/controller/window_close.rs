use std::sync::Arc;

use anyhow::Context;
use axum::debug_handler;
use axum::extract::{Path, State};
use serde::Serialize;

use crate::server::{ApiJson, ServiceClaim};
use crate::{AppError, AppState};

#[derive(Serialize)]
pub struct Response {}

#[debug_handler]
pub async fn controller_window_close(
    State(app_state): State<Arc<AppState>>,
    // claim: ServiceClaim,
    Path(id): Path<String>,
) -> anyhow::Result<ApiJson<Response>, AppError> {
    let id_usize: usize = id.parse().context("Invalid window ID")?;

    app_state.dispatcher.destroy(id_usize).context("Failed to destroy window")?;

    Ok(ApiJson(Response {}))
}
