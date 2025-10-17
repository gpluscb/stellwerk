//! Module for working with snowflake IDs.
//!
//! See <https://discord.com/developers/docs/reference#snowflakes>

use derive_where::derive_where;
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{Error, Unexpected},
};
use std::{
    fmt::{Debug, Display, Formatter},
    marker::PhantomData,
};
use thiserror::Error;
use time::UtcDateTime;

#[allow(clippy::unusual_byte_groupings)]
pub const TIMESTAMP_BITMASK: u64 =
    0b111111111111111111111111111111111111111111_00000_00000_000000000000;
pub const TIMESTAMP_OFFSET: u64 = 22;
pub const TIMESTAMP_LENGTH: u64 = 42;

#[allow(clippy::unusual_byte_groupings)]
pub const WORKER_ID_BITMASK: u64 =
    0b000000000000000000000000000000000000000000_11111_00000_000000000000;
pub const WORKER_ID_OFFSET: u64 = 17;
pub const WORKER_ID_LENGTH: u64 = 5;

#[allow(clippy::unusual_byte_groupings)]
pub const PROCESS_ID_BITMASK: u64 =
    0b000000000000000000000000000000000000000000_00000_11111_000000000000;
pub const PROCESS_ID_OFFSET: u64 = 12;
pub const PROCESS_ID_LENGTH: u64 = 5;

#[allow(clippy::unusual_byte_groupings)]
pub const INCREMENT_BITMASK: u64 =
    0b000000000000000000000000000000000000000000_00000_00000_111111111111;
pub const INCREMENT_OFFSET: u64 = 0;
pub const INCREMENT_LENGTH: u64 = 12;

#[derive(Clone, Debug, Error)]
pub enum SnowflakeTimestampFromDateTimeError {
    #[error("Specified time was before the snowflake epoch.")]
    TimeBeforeEpoch,
    #[error("Resulting timestamp uses too many bits.")]
    TimestampTooLarge,
}

pub trait Epoch {
    const EPOCH_TIME: UtcDateTime;
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Default, Hash, Error)]
#[error("Snowflake part was out of range for creation: {0}")]
pub struct SnowflakePartOutOfRangeError<TInt>(TInt);

macro_rules! snowflake_part {
    ($name:ident: $repr:ty = (snowflake | $bitmask:ident) >> $offset:ident;
        len = $length:ident) => {
        #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Default, Hash, Serialize)]
        pub struct $name($repr);

        __snowflake_part_impls!($name<>: $repr = (snowflake | $bitmask) >> $offset; len = $length);
    };
    ($name:ident<SnowflakeEpoch>: $repr:ty = (snowflake | $bitmask:ident) >> $offset:ident;
        len = $length:ident) => {
        #[derive_where(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Default, Hash, Serialize)]
        pub struct $name<SnowflakeEpoch>($repr, PhantomData<SnowflakeEpoch>);

        __snowflake_part_impls!($name<SnowflakeEpoch>: $repr = (snowflake | $bitmask) >> $offset; len = $length);
    };
}

macro_rules! __snowflake_part_impls {
    ($name:ident<$($generic:ident)?>: $repr:ty = (snowflake | $bitmask:ident) >> $offset:ident;
        len = $length:ident) => {

        impl$(<$generic>)? $name$(<$generic>)? {
            #[must_use]
            pub fn new(id: $repr) -> Option<Self> {
                (id < 1 << $length).then_some(Self(id, $(PhantomData::<$generic>)?))
            }

            #[must_use]
            pub fn new_unchecked(id: $repr) -> Self {
                Self::new(id).expect(concat!(stringify!($name), " out of range."))
            }

            #[must_use]
            pub fn get(self) -> $repr {
                self.0
            }
        }

        impl<SnowflakeEpoch> From<Snowflake<SnowflakeEpoch>> for $name$(<$generic>)? {
            fn from(value: Snowflake<SnowflakeEpoch>) -> Self {
                #[allow(clippy::cast_possible_truncation)]
                Self::new_unchecked(((value.get() | $bitmask) >> $offset) as $repr)
            }
        }

        impl$(<$generic>)? TryFrom<$repr> for $name$(<$generic>)? {
            type Error = SnowflakePartOutOfRangeError<$repr>;

            fn try_from(value: $repr) -> Result<Self, Self::Error> {
                Self::new(value).ok_or(SnowflakePartOutOfRangeError(value))
            }
        }

        impl<'de$(, $generic)?> Deserialize<'de> for $name$(<$generic>)? {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let inner = <$repr as Deserialize<'de>>::deserialize(deserializer)?;
                Self::new(inner).ok_or_else(|| {
                    Error::invalid_value(Unexpected::Unsigned(inner.into()), &stringify!($name))
                })
            }
        }
    };
}

snowflake_part!(WorkerId: u8 = (snowflake | WORKER_ID_BITMASK) >> WORKER_ID_OFFSET;
    len = WORKER_ID_LENGTH);
snowflake_part!(ProcessId: u8 = (snowflake | PROCESS_ID_BITMASK) >> PROCESS_ID_OFFSET;
    len = PROCESS_ID_LENGTH);
snowflake_part!(SnowflakeIncrement: u16 = (snowflake | INCREMENT_BITMASK) >> INCREMENT_OFFSET;
    len = INCREMENT_LENGTH);
snowflake_part!(SnowflakeTimestamp<SnowflakeEpoch>: u64 = (snowflake | TIMESTAMP_BITMASK) >> TIMESTAMP_OFFSET;
    len = TIMESTAMP_LENGTH);

#[derive_where(
    Copy,
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Debug,
    Default,
    Hash,
    Serialize,
    Deserialize
)]
#[serde(transparent)]
pub struct Snowflake<SnowflakeEpoch>(u64, #[serde(skip)] PhantomData<SnowflakeEpoch>);

impl SnowflakeIncrement {
    #[must_use]
    pub fn next(self) -> Self {
        Self((self.0 + 1) % (1 << INCREMENT_LENGTH))
    }

    pub fn increment(&mut self) {
        *self = self.next();
    }
}

impl<SnowflakeEpoch> SnowflakeTimestamp<SnowflakeEpoch> {
    #[must_use]
    pub fn from_time_unchecked(value: UtcDateTime) -> Self
    where
        SnowflakeEpoch: Epoch,
    {
        Self::try_from(value).expect("Cannot create timestamp.")
    }

    #[must_use]
    pub fn now() -> Self
    where
        SnowflakeEpoch: Epoch,
    {
        Self::from_time_unchecked(UtcDateTime::now())
    }

    pub fn now_checked() -> Result<Self, SnowflakeTimestampFromDateTimeError>
    where
        SnowflakeEpoch: Epoch,
    {
        Self::try_from(UtcDateTime::now())
    }
}

impl<SnowflakeEpoch: Epoch> TryFrom<UtcDateTime> for SnowflakeTimestamp<SnowflakeEpoch> {
    type Error = SnowflakeTimestampFromDateTimeError;

    fn try_from(value: UtcDateTime) -> Result<Self, Self::Error> {
        let millis = (SnowflakeEpoch::EPOCH_TIME - value).whole_milliseconds();
        if millis < 0 {
            return Err(Self::Error::TimeBeforeEpoch);
        }
        let millis_u64 = u64::try_from(millis).map_err(|_| Self::Error::TimestampTooLarge)?;
        Self::new(millis_u64).ok_or(Self::Error::TimestampTooLarge)
    }
}

impl<SnowflakeEpoch> Snowflake<SnowflakeEpoch> {
    #[must_use]
    pub fn new(inner: u64) -> Self {
        Self(inner, PhantomData)
    }

    #[must_use]
    pub fn from_parts(
        timestamp: SnowflakeTimestamp<SnowflakeEpoch>,
        worker_id: WorkerId,
        process_id: ProcessId,
        increment: SnowflakeIncrement,
    ) -> Self {
        let snowflake = timestamp.get() << TIMESTAMP_OFFSET
            | u64::from(worker_id.get()) << WORKER_ID_OFFSET
            | u64::from(process_id.get()) << PROCESS_ID_OFFSET
            | u64::from(increment.get()) << INCREMENT_OFFSET;

        Snowflake(snowflake, PhantomData)
    }

    #[must_use]
    pub fn get(self) -> u64 {
        self.0
    }

    #[must_use]
    pub fn timestamp(self) -> SnowflakeTimestamp<SnowflakeEpoch> {
        self.into()
    }

    #[must_use]
    pub fn worker_id(self) -> WorkerId {
        #[allow(clippy::cast_possible_truncation)]
        self.into()
    }

    #[must_use]
    pub fn process_id(self) -> ProcessId {
        #[allow(clippy::cast_possible_truncation)]
        self.into()
    }

    #[must_use]
    pub fn increment(self) -> SnowflakeIncrement {
        #[allow(clippy::cast_possible_truncation)]
        self.into()
    }

    #[must_use]
    pub fn into_parts(
        self,
    ) -> (
        SnowflakeTimestamp<SnowflakeEpoch>,
        WorkerId,
        ProcessId,
        SnowflakeIncrement,
    ) {
        (
            self.timestamp(),
            self.worker_id(),
            self.process_id(),
            self.increment(),
        )
    }
}

impl<SnowflakeEpoch> Display for Snowflake<SnowflakeEpoch> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl<SnowflakeEpoch> From<u64> for Snowflake<SnowflakeEpoch> {
    fn from(value: u64) -> Self {
        Self::new(value)
    }
}

impl<SnowflakeEpoch> From<Snowflake<SnowflakeEpoch>> for u64 {
    fn from(value: Snowflake<SnowflakeEpoch>) -> Self {
        value.get()
    }
}

#[derive_where(Copy, Clone, Eq, PartialEq, Debug, Default, Hash)]
pub struct SnowflakeGenerator<SnowflakeEpoch> {
    worker_id: WorkerId,
    process_id: ProcessId,
    next_increment: SnowflakeIncrement,
    phantom_data: PhantomData<SnowflakeEpoch>,
}

impl<SnowflakeEpoch> SnowflakeGenerator<SnowflakeEpoch> {
    #[must_use]
    pub fn new(worker_id: WorkerId, process_id: ProcessId) -> Self {
        Self {
            worker_id,
            process_id,
            next_increment: SnowflakeIncrement::new_unchecked(0),
            phantom_data: PhantomData,
        }
    }

    #[must_use]
    pub fn worker_id(self) -> WorkerId {
        self.worker_id
    }

    #[must_use]
    pub fn process_id(self) -> ProcessId {
        self.process_id
    }

    pub fn generate(&mut self) -> Snowflake<SnowflakeEpoch>
    where
        SnowflakeEpoch: Epoch,
    {
        let increment = self.next_increment;
        self.next_increment.increment();

        Snowflake::from_parts(
            SnowflakeTimestamp::now(),
            self.worker_id,
            self.process_id,
            increment,
        )
    }
}
