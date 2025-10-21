use crate::{AppError, AppState, server::ApiJson, webview::WryCreateOptions};
use axum::{debug_handler, extract::State};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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
    body: ApiJson<Request>,
) -> anyhow::Result<ApiJson<Response>, AppError> {
    let state_weak = app_state.wry.create(
        None,
        WryCreateOptions {
            title: body.0.title,
            url: body.0.url,
        },
    )?;

    let state = state_weak.upgrade().unwrap();

    Ok(ApiJson(Response { id: state.id }))
}
