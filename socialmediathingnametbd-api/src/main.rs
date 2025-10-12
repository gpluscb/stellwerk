use axum::routing::get;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
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

    let tracing_layer = TraceLayer::new_for_http();
    let app = axum::Router::new()
        .route("/", get(|| async { "hi" }))
        .layer(tracing_layer);

    let listener = tokio::net::TcpListener::bind("localhost:8080")
        .await
        .unwrap();
    axum::serve(listener, app).await.expect("Couldn't serve");
}
