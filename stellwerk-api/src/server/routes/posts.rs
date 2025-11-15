use crate::server::{Result, ServerError, ServerRouter, auth::AuthenticatedUser, json::Json};
use axum::extract::State;
use axum_extra::routing::{RouterExt, TypedPath};
use serde::Deserialize;
use std::sync::Arc;
use stellwerk_common::model::{
    Id,
    post::{PartialPost, Post, PostContent, PostMarker},
};
use stellwerk_db::client::DbClient;

pub fn routes() -> ServerRouter {
    ServerRouter::new()
        .typed_get(get_post)
        .typed_post(create_post)
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
        .ok_or(ServerError::PostByIdNotFound(id))?;

    Ok(Json(post))
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/posts/create", rejection(ServerError))]
struct CreatePostPath();

async fn create_post(
    CreatePostPath(): CreatePostPath,
    State(db): State<Arc<DbClient>>,
    user: AuthenticatedUser,
    Json(post): Json<PostContent>,
) -> Result<Json<PartialPost>> {
    let post = db.create_post(&post, user.user_id()).await?;

    Ok(Json(post))
}
