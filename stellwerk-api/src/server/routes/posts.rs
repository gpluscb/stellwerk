use crate::server::{Result, ServerError, ServerRouter, json::Json};
use axum::extract::State;
use axum_extra::routing::{RouterExt, TypedPath};
use serde::Deserialize;
use std::sync::Arc;
use stellwerk_common::model::{
    Id,
    post::{Post, PostMarker},
};
use stellwerk_db::client::DbClient;

pub fn routes() -> ServerRouter {
    ServerRouter::new().typed_get(get_post)
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
