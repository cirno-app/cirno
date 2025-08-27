#![feature(exit_status_error)]

use crate::{
    config::{EnvironmentState, load_config},
    daemon::ProcessDaemon,
    log::CombinedLogger,
    server::AppClaim,
};
use ::log::{debug, error, info};
use anyhow::{Context, Error, Result};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
};
use clap::{Args, Parser, Subcommand};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use serde_json::Value;
use std::{
    env::{args, current_exe},
    process::ExitCode,
    sync::{
        Arc, RwLock,
        atomic::{AtomicU64, Ordering},
    },
};
use tao::{event_loop::EventLoop, window::WindowBuilder};
use tap::Tap;
use thiserror::Error;
use tokio::spawn;
use tokio_util::sync::CancellationToken;
use wry::WebViewBuilder;

mod app;
mod config;
mod daemon;
mod log;
mod proc;
mod server;

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[command(flatten)]
    verbosity: Verbosity<InfoLevel>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Run(RunArgs),

    Start(StartArgs),

    Stop(StopArgs),
}

#[derive(Debug, Args)]
struct RunArgs {}

#[derive(Debug, Args)]
struct StartArgs {}

#[derive(Debug, Args)]
struct StopArgs {}

fn main() -> ExitCode {
    let logger = CombinedLogger::init();

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

    logger.push(Arc::new(
        env_logger::builder()
            .filter_level(cli.verbosity.into())
            .build(),
    ));

    info!("Cirno");

    let exe_path = current_exe().context("Failed to get current exe path")?;
    debug!("Executable: {}", exe_path.display());

    let exe_dir = exe_path.clone().tap_mut(|x| {
        x.pop();
    });
    debug!("Executable dir: {}", exe_dir.display());

    debug!("Arguments: {:?}", args().collect::<Vec<_>>());

    let env = load_config(exe_dir)
        .await
        .context("Failed to load config")?;

    match &cli.command {
        Commands::Run(_args) => {

            let app_state = Arc::new_cyclic(|app_state| {
                AppState {
                    env,

                    shutdown_token: CancellationToken::new(),

                    wry: WryStateRegistry::new(),

                    // As a daemon, ProcessDaemon will of course continue to exist until the program exits.
                    // Here we use Box::leak directly.
                    process_daemon: Box::leak(Box::new(ProcessDaemon::new(app_state.clone()))),
                }
            });

            app_state
                .process_daemon
                .init()
                .await
                .context("Failed to init process daemon")?;

            // Bind port
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
                .with_state(app_state.clone());

            let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
                .await
                .context("Failed to bind tcp port")?;

            // Write daemon.lock only after port bound

            // Start Server
            let app_state = app_state.clone();
            let shutdown_token = app_state.shutdown_token.clone();

            spawn(async move {
                let result = axum::serve(listener, app)
                    .with_graceful_shutdown(async move {
                        shutdown_token.cancelled().await;
                    })
                    .await;

                if let Result::Err(err) = result {
                    error!("{err}");
                    app_state.shutdown();
                }
            });
        }

        Commands::Start(_args) => {
        }

        Commands::Stop(_args) => {
        }
    }

    Ok(())
}

struct AppState {
    env: EnvironmentState,

    shutdown_token: CancellationToken,

    wry: WryStateRegistry,

    process_daemon: &'static mut ProcessDaemon,
}

impl AppState {
    fn shutdown(&self) {
        self.shutdown_token.cancel();
    }
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
    claim: AppClaim,
) -> Result<(StatusCode, Json<Value>), AppError> {
    (StatusCode::OK, Json(serde_json::json!({})))
}

async fn controller_app_backup(
    State(app_state): State<Arc<AppState>>,
    claim: AppClaim,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    (StatusCode::OK, Json(serde_json::json!({})))
}

async fn controller_app_start(
    State(app_state): State<Arc<AppState>>,
    claim: AppClaim,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    (StatusCode::OK, Json(serde_json::json!({})))
}

async fn controller_app_stop(
    State(app_state): State<Arc<AppState>>,
    claim: AppClaim,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    (StatusCode::OK, Json(serde_json::json!({})))
}

async fn controller_window_open(
    State(app_state): State<Arc<AppState>>,
    claim: AppClaim,
) -> Result<(StatusCode, Json<Value>), AppError> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .build(&event_loop)
        .context("Failed to create window")?;

    let builder = WebViewBuilder::new().with_url("https://tauri.app");

    #[cfg(not(target_os = "linux"))]
    let webview = builder.build(&window).context("Failed to create webview")?;
    #[cfg(target_os = "linux")]
    let webview = builder
        .build_gtk(window.gtk_window())
        .context("Failed to create webview")?;

    (StatusCode::OK, Json(serde_json::json!({})))
}

async fn controller_window_close(
    State(app_state): State<Arc<AppState>>,
    claim: AppClaim,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    (StatusCode::OK, Json(serde_json::json!({})))
}

async fn handler_notfound(
    State(app_state): State<Arc<AppState>>,
    claim: AppClaim,
) -> (StatusCode, [u8; 0]) {
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
