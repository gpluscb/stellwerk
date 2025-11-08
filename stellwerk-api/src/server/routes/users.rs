use crate::server::{Result, ServerError, ServerRouter, json::Json};
use axum::extract::State;
use axum_extra::routing::{RouterExt, TypedPath};
use serde::Deserialize;
use std::sync::Arc;
use stellwerk_common::model::{
    Id,
    post::PartialPost,
    user::{User, UserMarker},
};
use stellwerk_db::client::DbClient;

pub fn routes() -> ServerRouter {
    ServerRouter::new()
        .typed_get(get_user)
        .typed_get(get_user_posts)
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
        .ok_or(ServerError::UserByIdNotFound(id))?;

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
        .ok_or(ServerError::UserByIdNotFound(id))?;

    Ok(Json(posts))
}
