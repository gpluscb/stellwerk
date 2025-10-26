use crate::{
    model::{Id, user::UserMarker},
    util::PositiveDuration,
};
use argon2::{Argon2, Params};
use base64::{DecodeError, Engine, display::Base64Display, prelude::BASE64_STANDARD};
use std::{
    fmt::{Debug, Formatter},
    num::ParseIntError,
    str::FromStr,
};
use thiserror::Error;
use time::UtcDateTime;

pub const AUTH_TOKEN_CORE_LEN: usize = 24;
pub const AUTH_TOKEN_SALT_LEN: usize = 18;
pub const AUTH_TOKEN_HASH_LEN: usize = Params::DEFAULT_OUTPUT_LEN;

#[derive(Clone, Eq, PartialEq, Debug, Error)]
#[error("Hashing auth token failed: {0}")]
pub struct AuthTokenHashError(argon2::Error);

#[derive(Clone, Eq, PartialEq, Debug, Error)]
pub enum AuthTokenDecodeError {
    #[error("Not enough parts separated by ':'")]
    NotEnoughParts,
    #[error("Invalid user id: {0}")]
    InvalidUserId(ParseIntError),
    #[error("Decoding base64 failed: {0}")]
    Decode(#[from] DecodeError),
    #[error("The length of the core part is incorrect")]
    InvalidCoreLength,
    #[error("The length of the salt part is incorrect")]
    InvalidSaltLength,
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct AuthToken {
    pub user_id: Id<UserMarker>,
    pub core: [u8; AUTH_TOKEN_CORE_LEN],
    pub salt: [u8; AUTH_TOKEN_SALT_LEN],
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct AuthTokenHash(pub Box<[u8; AUTH_TOKEN_HASH_LEN]>);

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub struct Authentication {
    pub user: Id<UserMarker>,
    pub token_hash: AuthTokenHash,
    pub created_at: UtcDateTime,
    pub expires_after: Option<PositiveDuration>,
}

impl AuthToken {
    #[must_use]
    pub fn generate_random(user_id: Id<UserMarker>) -> Self {
        let core = rand::random();
        let salt = rand::random();

        Self {
            user_id,
            core,
            salt,
        }
    }

    #[must_use]
    pub fn as_token_str(&self) -> String {
        let user_id = self.user_id;
        let encoded_core = Base64Display::new(&self.core, &BASE64_STANDARD);
        let encoded_salt = Base64Display::new(&self.salt, &BASE64_STANDARD);

        format!("{user_id}:{encoded_core}:{encoded_salt}")
    }

    pub fn hash(&self) -> Result<AuthTokenHash, AuthTokenHashError> {
        let argon2 = Argon2::default();

        let mut hash = Box::new([0; AUTH_TOKEN_HASH_LEN]);
        argon2
            .hash_password_into(&self.core, &self.salt, &mut *hash)
            .map_err(AuthTokenHashError)?;

        Ok(AuthTokenHash(hash))
    }
}
impl FromStr for AuthToken {
    type Err = AuthTokenDecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.splitn(3, ':');

        let user_id_part = parts.next().ok_or(Self::Err::NotEnoughParts)?;
        let core_part = parts.next().ok_or(Self::Err::NotEnoughParts)?;
        let salt_part = parts.next().ok_or(Self::Err::NotEnoughParts)?;

        let user_id = u64::from_str(user_id_part)
            .map_err(Self::Err::InvalidUserId)?
            .into();
        let core = BASE64_STANDARD
            .decode(core_part)?
            .try_into()
            .map_err(|_| Self::Err::InvalidCoreLength)?;
        let salt = BASE64_STANDARD
            .decode(salt_part)?
            .try_into()
            .map_err(|_| Self::Err::InvalidSaltLength)?;

        Ok(Self {
            user_id,
            core,
            salt,
        })
    }
}

impl Debug for AuthToken {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthToken")
            .field("user_id", &self.user_id)
            .field("core", &"[redacted]")
            .field("salt", &"[redacted]")
            .finish()
    }
}

impl Debug for AuthTokenHash {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("AuthTokenHash").field(&"[redacted]").finish()
    }
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Default, Hash, Error)]
#[error("The auth token hash had an invalid length")]
pub struct InvalidAuthTokenHashError;

impl TryFrom<Box<[u8]>> for AuthTokenHash {
    type Error = InvalidAuthTokenHashError;

    fn try_from(value: Box<[u8]>) -> Result<Self, Self::Error> {
        Ok(Self(
            value.try_into().map_err(|_| InvalidAuthTokenHashError)?,
        ))
    }
}
