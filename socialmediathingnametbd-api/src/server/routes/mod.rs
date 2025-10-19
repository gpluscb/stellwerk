use crate::server::ServerRouter;
use axum::Router;

mod posts;
mod users;

pub fn routes() -> ServerRouter {
    Router::new().merge(posts::routes()).merge(users::routes())
}
