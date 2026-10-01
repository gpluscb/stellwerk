#![feature(duration_constructors)]

mod config;
mod login_logout;
mod oauth2;
mod open_api;
mod server;

use crate::{
    config::{ApiConfig, ReadConfigError, read_config},
    login_logout::LoginLogoutService,
    oauth2::{Oauth2Service, Oauth2SetupError},
    server::ServerState,
};
use std::{path::Path, sync::Arc};
use stellwerk_db::client::{DbClient, DbError};
use thiserror::Error;
use tokio::{signal, signal::unix::SignalKind, task::JoinError};
use tokio_util::sync::CancellationToken;
use tower_http::trace::TraceLayer;
use tracing::{debug, error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, Error)]
enum InitError {
    #[error("STELLWERK_API_CONFIG_FILE environment variable did not exist")]
    NoConfigEnvVar,
    #[error("Error loading configuration: {0}")]
    ConfigLoad(#[from] ReadConfigError),
    #[error("Error during OAuth2 setup: {0}")]
    Oauth2Setup(#[from] Oauth2SetupError),
    #[error("Error binding tcp listener: {0}")]
    TcpBind(std::io::Error),
    #[error("Error serving server: {0}")]
    TcpServe(std::io::Error),
    #[error("Error installing shutdown signal handler: {0}")]
    SignalHandler(std::io::Error),
    #[error("Database connection and migration failed: {0}")]
    DatabaseInitialization(DbError),
    #[error("A background task had issues: {0}")]
    Join(#[from] JoinError),
    #[error("Crypto provider installation failed")]
    CryptoProviderInstallation,
}

fn install_crypto_provider() -> Result<(), InitError> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .map_err(|_| InitError::CryptoProviderInstallation)?;

    Ok(())
}

fn install_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "stellwerk_api=debug,\
                stellwerk_common=debug,\
                stellwerk_db=debug,\
                tower_http=debug,axum::rejection=trace,sqlx=debug"
                    .into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

async fn connect_database(config: &ApiConfig) -> Result<DbClient, InitError> {
    DbClient::connect_and_migrate(&config.database_url, config.worker_id, config.process_id)
        .await
        .map_err(InitError::DatabaseInitialization)
}

async fn db_prune_loop(db: Arc<DbClient>, cancellation: CancellationToken) {
    loop {
        match db.drop_expired_tokens().await {
            Ok(dropped_rows) => debug!("Dropped {dropped_rows} expired tokens"),
            Err(error) => error!(%error, "Error trying to drop expired tokens"),
        }

        match db.drop_expired_oauth2_states().await {
            Ok(dropped_rows) => debug!("Dropped {dropped_rows} expired oauth2 states"),
            Err(error) => error!(%error, "Error trying to drop expired oauth2 states"),
        }

        if cancellation
            .run_until_cancelled(tokio::time::sleep(std::time::Duration::from_days(1)))
            .await
            .is_none()
        {
            return;
        }
    }
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
    install_crypto_provider()?;

    let config_path =
        std::env::var_os("STELLWERK_API_CONFIG_FILE").ok_or(InitError::NoConfigEnvVar)?;
    let config = read_config(Path::new(&config_path))?;

    let db_client = Arc::new(connect_database(&config).await?);
    let mut open_api = open_api::install_open_api();
    let oauth2_service = Oauth2Service::new(config.oauth2_providers_config)?;
    let login_logout_service = LoginLogoutService::new(config.login_logout_config);

    let tracing_layer = TraceLayer::new_for_http();
    let app = server::routes()
        .layer(tracing_layer)
        .finish_api(&mut open_api)
        .with_state(ServerState {
            db_client: Arc::clone(&db_client),
            open_api: Arc::new(open_api),
            oauth2_service: Arc::new(oauth2_service),
            login_logout_service: Arc::new(login_logout_service),
        });

    let server_address = config.server_address;
    let listener = tokio::net::TcpListener::bind(server_address)
        .await
        .map_err(InitError::TcpBind)?;
    info!("Listening on {server_address}");

    let cancellation_token = CancellationToken::new();
    let db_prune_loop_handle = tokio::spawn(db_prune_loop(db_client, cancellation_token.clone()));
    info!("Started database prune loop");

    axum::serve(listener, app)
        .with_graceful_shutdown(await_shutdown()?)
        .await
        .map_err(InitError::TcpServe)?;

    cancellation_token.cancel();
    db_prune_loop_handle.await?;

    Ok(())
}
