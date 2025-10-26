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
pub struct SocialmediathingnametbdEpoch;
impl Epoch for SocialmediathingnametbdEpoch {
    const EPOCH_TIME: UtcDateTime = utc_datetime!(2025-01-01 00:00);
}

pub type SocialmediathingnametbdSnowflake = Snowflake<SocialmediathingnametbdEpoch>;
pub type SocialmediathingnametbdSnowflakeGenerator =
    SnowflakeGenerator<SocialmediathingnametbdEpoch>;

#[derive(
    Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Default, Hash, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct Id<Marker>(
    SocialmediathingnametbdSnowflake,
    #[serde(skip)] PhantomData<Marker>,
);

impl<Marker> Id<Marker> {
    #[must_use]
    pub fn new(snowflake: SocialmediathingnametbdSnowflake) -> Self {
        Self(snowflake, PhantomData)
    }

    #[must_use]
    pub fn snowflake(self) -> SocialmediathingnametbdSnowflake {
        self.0
    }
}

impl<Marker> Display for Id<Marker> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl<Marker> From<SocialmediathingnametbdSnowflake> for Id<Marker> {
    fn from(value: SocialmediathingnametbdSnowflake) -> Self {
        Self::new(value)
    }
}

impl<Marker> From<Id<Marker>> for SocialmediathingnametbdSnowflake {
    fn from(value: Id<Marker>) -> Self {
        value.0
    }
}

impl<Marker> From<u64> for Id<Marker> {
    fn from(value: u64) -> Self {
        Id::new(SocialmediathingnametbdSnowflake::new(value))
    }
}

impl<Marker> From<Id<Marker>> for u64 {
    fn from(value: Id<Marker>) -> Self {
        value.snowflake().get()
    }
}
