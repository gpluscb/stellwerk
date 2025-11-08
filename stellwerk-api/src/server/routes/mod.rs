use crate::server::ServerRouter;

mod posts;
mod users;

pub fn routes() -> ServerRouter {
    ServerRouter::new()
        .merge(posts::routes())
        .merge(users::routes())
}
