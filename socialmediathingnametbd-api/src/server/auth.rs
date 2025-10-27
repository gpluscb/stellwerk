use crate::server::ServerError;
use axum::{
    extract::{FromRef, FromRequestParts},
    http::{StatusCode, request::Parts},
};
use axum_extra::{TypedHeader, typed_header::TypedHeaderRejection};
use headers::{Authorization, authorization::Bearer};
use socialmediathingnametbd_common::model::{
    Id,
    auth::{AuthToken, AuthTokenDecodeError, AuthTokenHashError},
    user::UserMarker,
};
use socialmediathingnametbd_db::client::DbClient;
use std::{hash::Hash, sync::Arc};
use thiserror::Error;
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

#[derive(Debug, Error)]
pub enum AuthenticationRejection {
    #[error("Authorization header was missing or invalid: {0}")]
    InvalidAuthorizationHeader(#[from] TypedHeaderRejection),
    #[error("The provided auth token could not be decoded: {0}")]
    AuthTokenFormat(#[from] AuthTokenDecodeError),
    #[error("Decoded user id from the token format did not match user id in database")]
    AuthTokenUserMismatch,
    #[error("The auth token could not be hashed: {0}")]
    AuthTokenHash(#[from] AuthTokenHashError),
    #[error("Provided token was invalid")]
    InvalidToken,
}

impl AuthenticationRejection {
    pub fn status(&self) -> StatusCode {
        match self {
            AuthenticationRejection::InvalidAuthorizationHeader(rejection) => {
                if rejection.is_missing() {
                    StatusCode::UNAUTHORIZED
                } else {
                    StatusCode::BAD_REQUEST
                }
            }
            AuthenticationRejection::AuthTokenFormat(_) => StatusCode::BAD_REQUEST,
            AuthenticationRejection::AuthTokenUserMismatch
            | AuthenticationRejection::InvalidToken => StatusCode::UNAUTHORIZED,
            AuthenticationRejection::AuthTokenHash(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
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
            .map_err(AuthenticationRejection::InvalidAuthorizationHeader)?
            .token()
            .parse()
            .map_err(AuthenticationRejection::from)?;

        let token_hash = request_token
            .hash()
            .map_err(AuthenticationRejection::from)?;

        let authentication = Arc::<DbClient>::from_ref(state)
            .fetch_auth(&token_hash)
            .await?
            .ok_or(AuthenticationRejection::InvalidToken)?;

        assert_eq!(authentication.token_hash, token_hash);

        if authentication.user != request_token.user_id {
            return Err(AuthenticationRejection::AuthTokenUserMismatch.into());
        }

        if let Some(expires_after) = authentication.expires_after
            && authentication.created_at + expires_after.get() < UtcDateTime::now()
        {
            return Err(AuthenticationRejection::InvalidToken.into());
        }

        Ok(Self {
            id: authentication.user,
        })
    }
}
