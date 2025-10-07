pub mod post;
pub mod user;

use crate::snowflake::{Epoch, Snowflake, SnowflakeGenerator};
use std::marker::PhantomData;
use time::UtcDateTime;
use time::macros::utc_datetime;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Default, Hash)]
pub struct SocialmediathingnametbdEpoch;
impl Epoch for SocialmediathingnametbdEpoch {
    const EPOCH_TIME: UtcDateTime = utc_datetime!(2025-01-01 00:00);
}

pub type SocialmediathingnametbdSnowflake = Snowflake<SocialmediathingnametbdEpoch>;
pub type SocialmediathingnametbdSnowflakeGenerator =
    SnowflakeGenerator<SocialmediathingnametbdEpoch>;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Default, Hash)]
pub struct Id<Marker>(SocialmediathingnametbdSnowflake, PhantomData<Marker>);

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
