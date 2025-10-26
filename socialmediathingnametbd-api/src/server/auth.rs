use crate::server::ServerError;
use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use axum_extra::TypedHeader;
use headers::{Authorization, authorization::Bearer};
use socialmediathingnametbd_common::model::{Id, auth::AuthToken, user::UserMarker};
use socialmediathingnametbd_db::client::DbClient;
use std::{hash::Hash, sync::Arc};
use time::UtcDateTime;

type AuthorizationHeader = TypedHeader<Authorization<Bearer>>;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Default, Hash)]
pub struct AuthenticatedUser {
    id: Id<UserMarker>,
}

impl AuthenticatedUser {
    #[must_use]
    pub fn user_id(self) -> Id<UserMarker> {
        self.id
    }
}

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    Arc<DbClient>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ServerError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let request_token: AuthToken = AuthorizationHeader::from_request_parts(parts, state)
            .await
            .map_err(ServerError::InvalidAuthorizationHeader)?
            .token()
            .parse()?;

        let token_hash = request_token.hash()?;

        let authentication = Arc::<DbClient>::from_ref(state)
            .fetch_auth(&token_hash)
            .await?
            .ok_or(ServerError::InvalidToken)?;

        assert_eq!(authentication.token_hash, token_hash);

        if let Some(expires_after) = authentication.expires_after
            && authentication.created_at + expires_after.get() < UtcDateTime::now()
        {
            return Err(ServerError::InvalidToken);
        }

        Ok(Self {
            id: authentication.user,
        })
    }
}
