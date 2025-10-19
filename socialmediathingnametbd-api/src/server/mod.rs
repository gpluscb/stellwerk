use axum::{
    Router,
    extract::{FromRef, rejection::PathRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use socialmediathingnametbd_common::model::{Id, post::PostMarker, user::UserMarker};
use socialmediathingnametbd_db::client::{DbClient, DbError};
use std::sync::Arc;
use thiserror::Error;
use tracing::error;

mod routes;

pub type ServerRouter = Router<ServerState>;

#[derive(Clone, Debug, FromRef)]
pub struct ServerState {
    pub db_client: Arc<DbClient>,
}

pub fn routes() -> ServerRouter {
    routes::routes()
}

pub type Result<T, E = ServerError> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("Path rejected: {0}")]
    PathRejection(#[from] PathRejection),
    #[error(transparent)]
    Database(#[from] DbError),
    #[error("Post with id {0} was not found.")]
    PostByIdNotFound(Id<PostMarker>),
    #[error("User with id {0} was not found.")]
    UserByIdNotFound(Id<UserMarker>),
}

impl ServerError {
    pub fn status(&self) -> StatusCode {
        match self {
            ServerError::PathRejection(_)
            | ServerError::PostByIdNotFound(_)
            | ServerError::UserByIdNotFound(_) => StatusCode::NOT_FOUND,
            ServerError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let status = self.status();

        error!(error = %self, %status, "Replying with error");

        (status, format!("Error: {status}")).into_response()
    }
}
