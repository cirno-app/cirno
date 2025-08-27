use crate::AppState;
use axum::{debug_handler, extract::State, http::StatusCode};
use std::sync::Arc;

#[debug_handler]
pub async fn handler_notfound(State(app_state): State<Arc<AppState>>) -> (StatusCode, [u8; 0]) {
    (StatusCode::NOT_FOUND, [])
}