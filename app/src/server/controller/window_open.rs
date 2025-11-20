use std::sync::Arc;

use anyhow::Context;
use axum::debug_handler;
use axum::extract::State;
use serde::{Deserialize, Serialize};

use crate::server::ApiJson;
use crate::ui_dispatcher::webview::WebViewCreateOptions;
use crate::{AppError, AppState};

#[derive(Deserialize)]
pub struct Request {
    title: String,
    url: String,
}

#[derive(Serialize)]
pub struct Response {
    id: usize,
}

#[debug_handler]
pub async fn controller_window_open(
    State(app_state): State<Arc<AppState>>,
    body: ApiJson<Request>,
) -> anyhow::Result<ApiJson<Response>, AppError> {
    let inst = app_state
        .dispatcher
        .create(
            None,
            WebViewCreateOptions {
                title: body.0.title,
                url: body.0.url,
            },
        )
        .context("Failed to create window")?;

    Ok(ApiJson(Response { id: inst.id }))
}
