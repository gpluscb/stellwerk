use axum::{
    Router,
    extract::{FromRef, rejection::PathRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use socialmediathingnametbd_db::client::{DbClient, DbError};
use std::sync::Arc;
use thiserror::Error;
use tracing::error;

mod posts;

pub type ServerRouter = Router<ServerState>;

#[derive(Clone, Debug, FromRef)]
pub struct ServerState {
    pub db_client: Arc<DbClient>,
}

pub fn routes() -> ServerRouter {
    Router::new().merge(posts::routes())
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Path rejected: {0}")]
    PathRejection(#[from] PathRejection),
    #[error(transparent)]
    Database(#[from] DbError),
}

impl Error {
    pub fn status(&self) -> StatusCode {
        match self {
            Error::PathRejection(_) => StatusCode::NOT_FOUND,
            Error::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status = self.status();

        error!(error = %self, %status, "Replying with error");

        format!("Error: {status}").into_response()
    }
}
