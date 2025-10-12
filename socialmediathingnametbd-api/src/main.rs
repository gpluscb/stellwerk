use axum::routing::get;
use serde::Deserialize;
use std::net::{IpAddr, SocketAddr};
use tower_http::trace::TraceLayer;
use tracing::debug;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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

fn get_env() -> Env {
    if let Err(e) = dotenvy::dotenv() {
        if e.not_found() {
            debug!("No .dotenv file found");
        } else {
            panic!("Error parsing .env file: {e}");
        }
    }

    envy::from_env().expect("Environment could not be parsed")
}

#[tokio::main]
async fn main() {
    install_tracing();
    let env = get_env();

    let tracing_layer = TraceLayer::new_for_http();
    let app = axum::Router::new()
        .route("/", get(|| async { "hi" }))
        .layer(tracing_layer);

    let server_address = SocketAddr::new(env.server_address, env.server_port);
    let listener = tokio::net::TcpListener::bind(server_address)
        .await
        .expect("Could not bind");
    axum::serve(listener, app).await.expect("Could not serve");
}
