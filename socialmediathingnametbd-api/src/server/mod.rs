use crate::server::auth::AuthenticationRejection;
use axum::{
    Router,
    extract::{
        FromRef, Request,
        rejection::{JsonRejection, PathRejection},
    },
    http::{StatusCode, Uri},
    response::{IntoResponse, Response},
};
use json::Json;
use serde::{Deserialize, Serialize};
use socialmediathingnametbd_common::model::{Id, post::PostMarker, user::UserMarker};
use socialmediathingnametbd_db::client::{DbClient, DbError};
use std::sync::Arc;
use thiserror::Error;
use tracing::error;

mod auth;
mod json;
mod routes;

pub type ServerRouter = Router<ServerState>;

#[derive(Clone, Debug, FromRef)]
pub struct ServerState {
    pub db_client: Arc<DbClient>,
}

pub fn routes() -> ServerRouter {
    routes::routes().fallback(fallback)
}

pub async fn fallback(request: Request) -> ServerError {
    ServerError::UnknownRoute(request.into_parts().0.uri)
}

pub type Result<T, E = ServerError> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("Unknown route requested: {0}")]
    UnknownRoute(Uri),
    #[error("Path rejected: {0}")]
    PathRejection(#[from] PathRejection),
    #[error("Incoming JSON rejected: {0}")]
    JsonRejection(#[from] JsonRejection),
    #[error("JSON response could not be serialized: {0}")]
    JsonResponse(#[from] serde_json::Error),
    #[error(transparent)]
    AuthenticationRejection(#[from] AuthenticationRejection),
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
            ServerError::AuthenticationRejection(rejection) => rejection.status(),
            ServerError::UnknownRoute(_)
            | ServerError::PathRejection(_)
            | ServerError::PostByIdNotFound(_)
            | ServerError::UserByIdNotFound(_) => StatusCode::NOT_FOUND,
            ServerError::JsonRejection(_) => StatusCode::BAD_REQUEST,
            ServerError::JsonResponse(_) | ServerError::Database(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
struct ErrorResponse {
    status: u16,
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let status = self.status();

        error!(error = %self, %status, "Replying with error");

        let error_response = ErrorResponse {
            status: status.as_u16(),
        };
        (status, Json(error_response)).into_response()
    }
}
