use crate::config::LoginLogoutConfig;
use stellwerk_common::{
    model::{
        auth::{AuthToken, AuthTokenHashError, Authentication},
        id::Id,
        user::UserMarker,
    },
    positive_duration::PositiveDuration,
};
use stellwerk_db::client::{DbClient, DbError};
use thiserror::Error;
use time::UtcDateTime;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct LoginData {
    pub expires_at: Option<UtcDateTime>,
    pub token: AuthToken,
}

#[derive(Debug, Error)]
pub enum LoginError {
    #[error(transparent)]
    TokenHash(#[from] AuthTokenHashError),
    #[error(transparent)]
    Database(#[from] DbError),
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Default, Hash)]
pub struct LoginLogoutService {
    pub expiring_token_duration: PositiveDuration,
}

impl LoginLogoutService {
    #[must_use]
    pub fn new(config: LoginLogoutConfig) -> Self {
        Self {
            expiring_token_duration: config.expiring_token_duration_seconds,
        }
    }

    pub async fn login_user(
        &self,
        db: &DbClient,
        user_id: Id<UserMarker>,
        expires: bool,
    ) -> Result<LoginData, LoginError> {
        // Generate new api token for user
        let random_token = AuthToken::generate_random(user_id);
        let hash = random_token.hash()?;

        let created_at = UtcDateTime::now();
        let expires_after = expires.then_some(self.expiring_token_duration);

        let authentication = Authentication {
            user: user_id,
            token_hash: hash,
            created_at,
            expires_after,
        };

        // Store newly created authentication
        db.create_auth(&authentication).await?;

        let expires_at = expires_after.map(|expires_after| created_at + expires_after.get());

        Ok(LoginData {
            expires_at,
            token: random_token,
        })
    }
}
