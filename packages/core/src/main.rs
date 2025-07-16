use anyhow::{Error, Result};
use axum::{
    Json, Router,
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
};
use clap::{Args, Parser, Subcommand};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tao::{event_loop::EventLoop, window::WindowBuilder};
use wry::WebViewBuilder;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Run(RunArgs),

    Start(StartArgs),

    Stop(StopArgs),
}

#[derive(Args)]
struct RunArgs {}

#[derive(Args)]
struct StartArgs {}

#[derive(Args)]
struct StopArgs {}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Run(_args) => {
            let api_routes = Router::new()
                .route("/gc", post(controller_gc))
                .route("/app/{id}/backup", post(controller_app_backup))
                .route("/app/{id}/start", post(controller_app_start))
                .route("/app/{id}/stop", post(controller_app_stop))
                .route("/window/open", post(controller_window_open))
                .route("/window/{id}/close", post(controller_window_close));

            let app = Router::new()
                .nest("/api/v1", api_routes)
                .fallback(handler_notfound);

            let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;

            axum::serve(listener, app).await?;
        }

        Commands::Start(_args) => {
        }

        Commands::Stop(_args) => {
        }
    }

    Ok(())
}

struct AppError(Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, "").into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

async fn controller_gc() -> Result<(StatusCode, Json<Value>), AppError> {
    (StatusCode::OK, Json(serde_json::json!({})))
}

async fn controller_app_backup(
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    (StatusCode::OK, Json(serde_json::json!({})))
}

async fn controller_app_start(
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    (StatusCode::OK, Json(serde_json::json!({})))
}

async fn controller_app_stop(
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    (StatusCode::OK, Json(serde_json::json!({})))
}

async fn controller_window_open() -> Result<(StatusCode, Json<Value>), AppError> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop)?;

    let builder = WebViewBuilder::new().with_url("https://tauri.app");

    #[cfg(not(target_os = "linux"))]
    let webview = builder.build(&window)?;
    #[cfg(target_os = "linux")]
    let webview = builder.build_gtk(window.gtk_window())?;

    (StatusCode::OK, Json(serde_json::json!({})))
}

async fn controller_window_close(
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    (StatusCode::OK, Json(serde_json::json!({})))
}

async fn handler_notfound() -> (StatusCode, [u8; 0]) {
    (StatusCode::NOT_FOUND, [])
}
