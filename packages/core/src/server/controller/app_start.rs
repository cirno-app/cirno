use std::sync::Arc;

use axum::debug_handler;
use axum::extract::{Path, State};
use serde::Serialize;

use crate::server::ApiJson;
use crate::{AppError, AppState};

#[derive(Serialize)]
pub struct Response {}

#[debug_handler]
pub async fn controller_app_start(
    State(app_state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> anyhow::Result<ApiJson<Response>, AppError> {
    app_state.process_daemon.start(&id).await?;

    Ok(ApiJson(Response {}))
}
