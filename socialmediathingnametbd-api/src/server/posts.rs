use crate::server::{Error as ServerError, Result, ServerRouter};
use axum::{Json, Router, extract::State, http::StatusCode};
use axum_extra::routing::{RouterExt, TypedPath};
use serde::Deserialize;
use socialmediathingnametbd_common::model::{
    Id,
    post::{Post, PostMarker},
};
use socialmediathingnametbd_db::client::DbClient;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Post with id {0} was not found.")]
    PostByIdNotFound(Id<PostMarker>),
}

impl Error {
    pub fn status(&self) -> StatusCode {
        match self {
            Error::PostByIdNotFound(_) => StatusCode::NOT_FOUND,
        }
    }
}

pub fn routes() -> ServerRouter {
    Router::new().typed_get(get_post)
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/posts/{id}", rejection(ServerError))]
struct GetPostPath {
    id: Id<PostMarker>,
}

async fn get_post(
    GetPostPath { id }: GetPostPath,
    State(db): State<Arc<DbClient>>,
) -> Result<Json<Post>> {
    let post = db
        .fetch_post(id)
        .await?
        .ok_or(Error::PostByIdNotFound(id))?;

    Ok(Json(post))
}
