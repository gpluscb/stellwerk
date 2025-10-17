use crate::server::{Error as ServerError, Result, ServerRouter};
use axum::{Json, Router, extract::State, http::StatusCode};
use axum_extra::routing::{RouterExt, TypedPath};
use serde::Deserialize;
use socialmediathingnametbd_common::model::{
    Id,
    post::PartialPost,
    user::{User, UserMarker},
};
use socialmediathingnametbd_db::client::DbClient;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("User with id {0} was not found.")]
    UserByIdNotFound(Id<UserMarker>),
}

impl Error {
    pub fn status(&self) -> StatusCode {
        match self {
            Error::UserByIdNotFound(_) => StatusCode::NOT_FOUND,
        }
    }
}

pub fn routes() -> ServerRouter {
    Router::new().typed_get(get_user).typed_get(get_user_posts)
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/users/{id}", rejection(ServerError))]
struct GetUserPath {
    id: Id<UserMarker>,
}

async fn get_user(
    GetUserPath { id }: GetUserPath,
    State(db): State<Arc<DbClient>>,
) -> Result<Json<User>> {
    let user = db
        .fetch_user(id)
        .await?
        .ok_or(Error::UserByIdNotFound(id))?;

    Ok(Json(user))
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/users/{id}/posts", rejection(ServerError))]
struct GetUserPostsPath {
    id: Id<UserMarker>,
}

async fn get_user_posts(
    GetUserPostsPath { id }: GetUserPostsPath,
    State(db): State<Arc<DbClient>>,
) -> Result<Json<Vec<PartialPost>>> {
    let posts = db
        .fetch_user_posts(id)
        .await?
        .ok_or(Error::UserByIdNotFound(id))?;

    Ok(Json(posts))
}
