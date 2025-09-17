#![feature(exit_status_error)]

use crate::{
    config::{EnvironmentState, load_config},
    daemon::ProcessDaemon,
    log::CombinedLogger,
    server::controller::{
        app_backup::controller_app_backup, app_start::controller_app_start,
        app_stop::controller_app_stop, gc::controller_gc, notfound::handler_notfound,
        window_close::controller_window_close, window_open::controller_window_open,
    },
    ui_dispatcher::Dispatcher,
    webview::WryStateRegistry,
};
use ::log::{debug, error, info};
use anyhow::{Context, Error, Result};
use axum::{
    Router,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
};
use clap::{Args, Parser, Subcommand};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use std::{
    env::{args, current_exe},
    process::ExitCode,
    sync::Arc,
};
use tap::Tap;
use tokio::{select, spawn, sync::Notify};
use tokio_util::sync::CancellationToken;

mod app;
mod config;
mod daemon;
mod log;
mod proc;
mod server;
mod ui_dispatcher;
mod webview;
mod window;

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

            // let shutdown_notify = Arc::new(Notify::const_new());

            let (dispatcher_init, dispatcher) = Dispatcher::new();

            let app_state = AppState::new(
                env, // &shutdown_notify,
                dispatcher,
            );

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
            let shutdown_token = CancellationToken::new();
            let server_shutdown_token = shutdown_token.clone();

            spawn(async move {
                let result = axum::serve(listener, app)
                    .with_graceful_shutdown(async move {
                        server_shutdown_token.cancelled().await;
                    })
                    .await;

                if let Err(err) = result {
                    error!("{err}");
                    app_state.shutdown();
                }
            });

            dispatcher_init.run();

            // Graceful shutdown on main thread
            // graceful_shutdown(shutdown_notify, shutdown_token).await;
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

    // shutdown_notify: Arc<Notify>,
    dispatcher: Dispatcher,

    wry: WryStateRegistry,

    process_daemon: ProcessDaemon,
}

impl AppState {
    fn new(
        env: EnvironmentState,
        // shutdown_notify: &Arc<Notify>,
        dispatcher: Dispatcher,
    ) -> Arc<AppState> {
        Arc::new_cyclic(|app_state| {
            AppState {
                env,

                // shutdown_notify: shutdown_notify.clone(),
                dispatcher,

                wry: WryStateRegistry::new(app_state.clone()),

                process_daemon: ProcessDaemon::new(app_state.clone()),
            }
        })
    }

    fn shutdown(&self) {
        // self.shutdown_notify.notify_one();
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

async fn graceful_shutdown(shutdown_notify: Arc<Notify>, shutdown_token: CancellationToken) {
    // 1. Wait for shutdown signals
    let signal = shutdown_signal_received(shutdown_notify).await;

    info!("{} signal received, starting to shutdown", signal);

    // 2. Tell all components to shutdown
    shutdown_token.cancel();
}

async fn shutdown_signal_received(shutdown_notify: Arc<Notify>) -> &'static str {
    #[cfg(target_os = "windows")]
    select! {
        // Some signals do not work on Windows 7.
        // Fill Err arm with std::future::pending.
        //
        // SIGQUIT
        _ = async {
            match tokio::signal::windows::ctrl_break() {
                Ok(mut signal) => signal.recv().await,
                Err(_) => std::future::pending().await,
            }
        } => "SIGQUIT",
        // SIGINT
        _ = async {
            match tokio::signal::windows::ctrl_c() {
                Ok(mut signal) => signal.recv().await,
                Err(_) => std::future::pending().await,
            }
        } => "SIGINT",
        // SIGTERM, "the normal way to politely ask a program to terminate"
        _ = async {
            match tokio::signal::windows::ctrl_close() {
                Ok(mut signal) => signal.recv().await,
                Err(_) => std::future::pending().await,
            }
        } => "SIGTERM",
        _ = async {
            match tokio::signal::windows::ctrl_logoff() {
                Ok(mut signal) => signal.recv().await,
                Err(_) => std::future::pending().await,
            }
        } => "Logoff",
        _ = async {
            match tokio::signal::windows::ctrl_shutdown() {
                Ok(mut signal) => signal.recv().await,
                Err(_) => std::future::pending().await,
            }
        } => "Shutdown",
        _ = shutdown_notify.notified() => "Shutdown",
    }

    #[cfg(not(target_os = "windows"))]
    select! {
        // SIGTERM, "the normal way to politely ask a program to terminate"
        _ = async {
            match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
                Ok(mut signal) => signal.recv().await,
                Err(_) => std::future::pending().await,
            }
        } => "SIGTERM",
        // SIGINT, Ctrl-C
        _ = async {
            match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt()) {
                Ok(mut signal) => signal.recv().await,
                Err(_) => std::future::pending().await,
            }
        } => "SIGINT",
        // SIGQUIT, Ctrl-\
        _ = async {
            match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::quit()) {
                Ok(mut signal) => signal.recv().await,
                Err(_) => std::future::pending().await,
            }
        } => "SIGQUIT",
        // SIGHUP, Terminal disconnected. SIGHUP also needs gracefully terminating
        _ = async {
            match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::hangup()) {
                Ok(mut signal) => signal.recv().await,
                Err(_) => std::future::pending().await,
            }
        } => "SIGHUP",
        _ = shutdown_notify.notified() => "Shutdown",
    }
}
