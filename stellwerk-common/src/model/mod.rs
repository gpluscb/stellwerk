pub mod auth;
pub mod post;
pub mod user;

use crate::{
    model::{auth::InvalidAuthTokenHashError, user::InvalidUserHandleError},
    snowflake::{Epoch, Snowflake, SnowflakeGenerator},
    util::NonPositiveDurationError,
};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, marker::PhantomData};
use thiserror::Error;
use time::{UtcDateTime, macros::utc_datetime};

#[derive(Clone, Eq, PartialEq, Debug, Hash, Error)]
pub enum ModelValidationError {
    #[error(transparent)]
    UserHandle(#[from] InvalidUserHandleError),
    #[error(transparent)]
    NonPositiveDuration(#[from] NonPositiveDurationError),
    #[error(transparent)]
    TokenHash(#[from] InvalidAuthTokenHashError),
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Default, Hash)]
pub struct StellwerkEpoch;
impl Epoch for StellwerkEpoch {
    const EPOCH_TIME: UtcDateTime = utc_datetime!(2025-01-01 00:00);
}

pub type StellwerkSnowflake = Snowflake<StellwerkEpoch>;
pub type StellwerkSnowflakeGenerator = SnowflakeGenerator<StellwerkEpoch>;

#[derive(
    Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Default, Hash, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct Id<Marker>(StellwerkSnowflake, #[serde(skip)] PhantomData<Marker>);

impl<Marker> Id<Marker> {
    #[must_use]
    pub fn new(snowflake: StellwerkSnowflake) -> Self {
        Self(snowflake, PhantomData)
    }

    #[must_use]
    pub fn snowflake(self) -> StellwerkSnowflake {
        self.0
    }
}

impl<Marker> Display for Id<Marker> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl<Marker> From<StellwerkSnowflake> for Id<Marker> {
    fn from(value: StellwerkSnowflake) -> Self {
        Self::new(value)
    }
}

impl<Marker> From<Id<Marker>> for StellwerkSnowflake {
    fn from(value: Id<Marker>) -> Self {
        value.0
    }
}

impl<Marker> From<u64> for Id<Marker> {
    fn from(value: u64) -> Self {
        Id::new(StellwerkSnowflake::new(value))
    }
}

impl<Marker> From<Id<Marker>> for u64 {
    fn from(value: Id<Marker>) -> Self {
        value.snowflake().get()
    }
}
