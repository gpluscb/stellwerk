use axum::routing::get;
use serde::Deserialize;
use std::net::{IpAddr, SocketAddr};
use thiserror::Error;
use tokio::{signal, signal::unix::SignalKind};
use tower_http::trace::TraceLayer;
use tracing::{debug, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, Error)]
enum InitError {
    #[error("Error parsing .env file: {0}")]
    Dotenv(#[from] dotenvy::Error),
    #[error("Error parsing environment: {0}")]
    Envy(#[from] envy::Error),
    #[error("Error binding tcp listener: {0}")]
    TcpBind(std::io::Error),
    #[error("Error serving server: {0}")]
    TcpServe(std::io::Error),
    #[error("Error installing shutdown signal handler: {0}")]
    SignalHandler(std::io::Error),
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Deserialize)]
struct Env {
    server_address: IpAddr,
    server_port: u16,
}

fn install_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "socialmediathingnametbd_api=debug,\
                socialmediathingnametbd_common=debug,\
                tower_http=debug,axum::rejection=trace,sqlx=debug"
                    .into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

fn get_env() -> Result<Env, InitError> {
    if let Err(e) = dotenvy::dotenv() {
        if e.not_found() {
            debug!("No .env file found");
        } else {
            return Err(e.into());
        }
    }

    envy::from_env().map_err(InitError::from)
}

fn await_shutdown() -> Result<impl Future<Output = ()>, InitError> {
    #[cfg(unix)]
    let mut ctrl_c_signal =
        signal::unix::signal(SignalKind::interrupt()).map_err(InitError::SignalHandler)?;

    #[cfg(not(unix))]
    let mut ctrl_c_signal = signal::windows::ctrl_c().map_err(InitError::SignalHandler)?;

    #[cfg(unix)]
    let mut terminate_signal =
        signal::unix::signal(SignalKind::terminate()).map_err(InitError::SignalHandler)?;

    #[cfg(not(unix))]
    let terminate_future = std::future::pending::<()>();

    Ok(async move {
        #[cfg(unix)]
        let terminate_future = terminate_signal.recv();

        tokio::select! {
            _ = ctrl_c_signal.recv() => {},
            _ = terminate_future => {},
        }

        info!("Shutdown signal received");
    })
}

#[tokio::main]
async fn main() -> Result<(), InitError> {
    install_tracing();
    let env = get_env()?;

    let tracing_layer = TraceLayer::new_for_http();
    let app = axum::Router::new()
        .route("/", get(|| async { "hi" }))
        .layer(tracing_layer);

    let server_address = SocketAddr::new(env.server_address, env.server_port);
    let listener = tokio::net::TcpListener::bind(server_address)
        .await
        .map_err(InitError::TcpBind)?;
    axum::serve(listener, app)
        .with_graceful_shutdown(await_shutdown()?)
        .await
        .map_err(InitError::TcpServe)?;

    Ok(())
}
