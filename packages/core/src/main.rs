use crate::log::combined_logger::CombinedLogger;
use ::log::{LevelFilter, error};
use anyhow::{Error, Result};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
};
use clap::{Args, Parser, Subcommand};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::{Arc, RwLock, atomic::Ordering};
use std::{process::ExitCode, sync::atomic::AtomicU64};
use tao::{event_loop::EventLoop, window::WindowBuilder};
use thiserror::Error;
use wry::WebViewBuilder;

mod log;

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

fn main() -> ExitCode {
    let logger = CombinedLogger::init();

    logger.push(Arc::new(
        env_logger::builder()
            .filter_level(LevelFilter::Info)
            .build(),
    ));

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(main_async(logger))
}

async fn main_async(logger: Arc<CombinedLogger>) -> ExitCode {
    match main_async_intl(logger).await {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            error!("{err:?}");
            ExitCode::FAILURE
        }
    }
}

async fn main_async_intl(logger: Arc<CombinedLogger>) -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Run(_args) => {s
            let app_state = Arc::new(AppState {
                wry: WryStateRegistry::new(),
            });

            let api_routes = Router::new()
                .route("/gc", post(controller_gc))
                .route("/app/{id}/backup", post(controller_app_backup))
                .route("/app/{id}/start", post(controller_app_start))
                .route("/app/{id}/stop", post(controller_app_stop))
                .route("/window/open", post(controller_window_open))
                .route("/window/{id}/close", post(controller_window_close));

            let app = Router::new()
                .nest("/api/v1", api_routes)
                .fallback(handler_notfound)
                .with_state(app_state);

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

struct AppState {
    wry: WryStateRegistry,
}

struct AppError(Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Error: {}", self.0),
        )
            .into_response()
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

async fn controller_gc(
    State(app_state): State<Arc<AppState>>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    (StatusCode::OK, Json(serde_json::json!({})))
}

async fn controller_app_backup(
    State(app_state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    (StatusCode::OK, Json(serde_json::json!({})))
}

async fn controller_app_start(
    State(app_state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    (StatusCode::OK, Json(serde_json::json!({})))
}

async fn controller_app_stop(
    State(app_state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    (StatusCode::OK, Json(serde_json::json!({})))
}

async fn controller_window_open(
    State(app_state): State<Arc<AppState>>,
) -> Result<(StatusCode, Json<Value>), AppError> {
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
    State(app_state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    (StatusCode::OK, Json(serde_json::json!({})))
}

async fn handler_notfound(State(app_state): State<Arc<AppState>>) -> (StatusCode, [u8; 0]) {
    (StatusCode::NOT_FOUND, [])
}

// region: WryState

#[derive(Debug, Error)]
enum WryStateRegistryError {
    #[error("Registry is full, no available IDs")]
    RegistryFull,
    #[error("Invalid ID: {0} (must be 0-63)")]
    InvalidId(u8),
    #[error("No window found for ID: {0}")]
    WindowNotFound(u8),
}

struct WryState {}

struct WryStateRegistry {
    intl: RwLock<WryStateRegistryIntl>,
}

struct WryStateRegistryIntl {
    map: AtomicU64,
    reg: [Option<Arc<RwLock<WryState>>>; 64],
}

impl WryStateRegistry {
    pub fn new() -> Self {
        Self {
            intl: RwLock::new(WryStateRegistryIntl {
                map: AtomicU64::new(0),
                reg: [(); 64].map(|_| None),
            }),
        }
    }

    pub fn create(
        &self,
        state: WryState,
    ) -> Result<(u8, Arc<RwLock<WryState>>), WryStateRegistryError> {
        let mut intl = self.intl.write().unwrap();
        let bitmap = intl.map.load(Ordering::Acquire);
        let free_bit = (0..64).find(|i| (bitmap & (1 << i)) == 0);

        match free_bit {
            Some(id) => {
                let arc = Arc::new(RwLock::new(state));
                intl.reg[id] = Some(arc.clone());

                Ok((id as u8, arc))
            }
            None => Err(WryStateRegistryError::RegistryFull),
        }
    }

    pub fn get(&self, id: u8) -> Result<Arc<RwLock<WryState>>, WryStateRegistryError> {
        let intl = self.intl.read().unwrap();

        if id >= 64 {
            return Err(WryStateRegistryError::InvalidId(id));
        }

        intl.reg[id as usize]
            .as_ref()
            .map(Arc::clone)
            .ok_or(WryStateRegistryError::WindowNotFound(id))
    }

    pub fn destroy(&self, id: u8) -> Result<(), WryStateRegistryError> {
        let mut intl = self.intl.write().unwrap();

        if id >= 64 {
            return Err(WryStateRegistryError::InvalidId(id));
        }

        let state = intl.reg[id as usize].take();

        let Some(state) = state else {
            return Err(WryStateRegistryError::WindowNotFound(id));
        };

        intl.map.fetch_and(!(1 << id), Ordering::AcqRel);

        Ok(())
    }
}

// endregion
