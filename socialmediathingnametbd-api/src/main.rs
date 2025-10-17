mod server;

use crate::server::ServerState;
use serde::Deserialize;
use socialmediathingnametbd_common::snowflake::{ProcessId, WorkerId};
use socialmediathingnametbd_db::client::{DbClient, DbError};
use std::{
    net::{IpAddr, SocketAddr},
    sync::Arc,
};
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
    #[error("Database connection and migration failed: {0}")]
    DatabaseInitialization(DbError),
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Deserialize)]
struct Env {
    server_address: IpAddr,
    server_port: u16,
    database_url: Box<str>,
    worker_id: WorkerId,
    process_id: ProcessId,
}

fn install_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "socialmediathingnametbd_api=debug,\
                socialmediathingnametbd_common=debug,\
                socialmediathingnametbd_db=debug,\
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

async fn init_state(env: &Env) -> Result<ServerState, InitError> {
    let db_client = DbClient::connect_and_migrate(&env.database_url, env.worker_id, env.process_id)
        .await
        .map_err(InitError::DatabaseInitialization)?;

    Ok(ServerState {
        db_client: Arc::new(db_client),
    })
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

    let state = init_state(&env).await?;
    let tracing_layer = TraceLayer::new_for_http();
    let app = server::routes().layer(tracing_layer).with_state(state);

    let server_address = SocketAddr::new(env.server_address, env.server_port);
    let listener = tokio::net::TcpListener::bind(server_address)
        .await
        .map_err(InitError::TcpBind)?;
    info!("Listening on {server_address}");
    axum::serve(listener, app)
        .with_graceful_shutdown(await_shutdown()?)
        .await
        .map_err(InitError::TcpServe)?;

    Ok(())
}
