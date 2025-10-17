use crate::server::{Error, Result, ServerRouter};
use axum::{Json, Router, extract::State};
use axum_extra::routing::{RouterExt, TypedPath};
use serde::Deserialize;
use socialmediathingnametbd_common::model::{
    Id,
    post::{Post, PostMarker},
};
use socialmediathingnametbd_db::client::DbClient;
use std::sync::Arc;

pub fn routes() -> ServerRouter {
    Router::new().typed_get(get_post)
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/posts/{id}", rejection(Error))]
struct GetPostPath {
    id: Id<PostMarker>,
}

#[axum::debug_handler]
async fn get_post(
    GetPostPath { id }: GetPostPath,
    State(db): State<Arc<DbClient>>,
) -> Result<Json<Post>> {
    let post_option = db.fetch_post(id).await?;

    Ok(Json(post_option.unwrap()))
}
