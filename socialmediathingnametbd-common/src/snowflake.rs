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
use time::{Duration, UtcDateTime};

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Hash, Error)]
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
    ($name:ident: $repr:ty = snowflake & $bitmask:literal) => {
        #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Default, Hash, Serialize)]
        pub struct $name($repr);

        __snowflake_part_impls!($name<>: $repr = snowflake & $bitmask);
    };
    ($name:ident<SnowflakeEpoch>: $repr:ty = snowflake & $bitmask:literal) => {
        #[derive_where(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Default, Hash, Serialize)]
        pub struct $name<SnowflakeEpoch>($repr, PhantomData<SnowflakeEpoch>);

        __snowflake_part_impls!($name<SnowflakeEpoch>: $repr = snowflake & $bitmask);
    };
}

macro_rules! __snowflake_part_impls {
    ($name:ident<$($generic:ident)?>: $repr:ty = snowflake & $bitmask:literal) => {

        impl$(<$generic>)? $name$(<$generic>)? {
            const SNOWFLAKE_BITMASK: u64 = $bitmask;
            const SNOWFLAKE_OFFSET: u64 = Self::SNOWFLAKE_BITMASK.trailing_zeros() as u64;
            const BIT_COUNT: u64 = Self::SNOWFLAKE_BITMASK.count_ones() as u64;
            const MAX_VALUE: $repr = (1 << Self::BIT_COUNT) - 1;

            #[must_use]
            pub fn new(value: $repr) -> Option<Self> {
                (value <= Self::MAX_VALUE).then_some(Self(value, $(PhantomData::<$generic>)?))
            }

            #[must_use]
            pub fn new_unchecked(value: $repr) -> Self {
                Self::new(value).expect(concat!(stringify!($name), " out of range."))
            }

            #[must_use]
            pub fn get(self) -> $repr {
                self.0
            }
        }

        impl<SnowflakeEpoch> From<Snowflake<SnowflakeEpoch>> for $name$(<$generic>)? {
            fn from(value: Snowflake<SnowflakeEpoch>) -> Self {
                #[allow(clippy::cast_possible_truncation)]
                Self::new_unchecked(
                    ((value.get() & Self::SNOWFLAKE_BITMASK) >> Self::SNOWFLAKE_OFFSET) as $repr
                )
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

snowflake_part!(WorkerId: u8 = snowflake & 0x0000_0000_003E_0000);
snowflake_part!(ProcessId: u8 = snowflake & 0x0000_0000_0001_F000);
snowflake_part!(SnowflakeIncrement: u16 = snowflake & 0x0000_0000_0000_0FFF);
snowflake_part!(SnowflakeTimestamp<SnowflakeEpoch>: u64 = snowflake & 0xFFFF_FFFF_FFC0_0000);

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
        Self((self.0 + 1) % (1 << Self::BIT_COUNT))
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
        let millis = (value - SnowflakeEpoch::EPOCH_TIME).whole_milliseconds();
        if millis < 0 {
            return Err(Self::Error::TimeBeforeEpoch);
        }
        let millis_u64 = u64::try_from(millis).map_err(|_| Self::Error::TimestampTooLarge)?;
        Self::new(millis_u64).ok_or(Self::Error::TimestampTooLarge)
    }
}

impl<SnowflakeEpoch: Epoch> From<SnowflakeTimestamp<SnowflakeEpoch>> for UtcDateTime {
    fn from(value: SnowflakeTimestamp<SnowflakeEpoch>) -> Self {
        SnowflakeEpoch::EPOCH_TIME
            + Duration::milliseconds(value.0.try_into().expect("Invalid timestamp value"))
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
        let snowflake = timestamp.get() << SnowflakeTimestamp::<SnowflakeEpoch>::SNOWFLAKE_OFFSET
            | u64::from(worker_id.get()) << WorkerId::SNOWFLAKE_OFFSET
            | u64::from(process_id.get()) << ProcessId::SNOWFLAKE_OFFSET
            | u64::from(increment.get()) << SnowflakeIncrement::SNOWFLAKE_OFFSET;

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
        self.into()
    }

    #[must_use]
    pub fn process_id(self) -> ProcessId {
        self.into()
    }

    #[must_use]
    pub fn increment(self) -> SnowflakeIncrement {
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

    pub fn generate_at(&mut self, time: UtcDateTime) -> Snowflake<SnowflakeEpoch>
    where
        SnowflakeEpoch: Epoch,
    {
        let increment = self.next_increment;
        self.next_increment.increment();

        Snowflake::from_parts(
            SnowflakeTimestamp::from_time_unchecked(time),
            self.worker_id,
            self.process_id,
            increment,
        )
    }

    pub fn generate(&mut self) -> Snowflake<SnowflakeEpoch>
    where
        SnowflakeEpoch: Epoch,
    {
        self.generate_at(UtcDateTime::now())
    }
}

#[cfg(test)]
mod tests {
    use crate::snowflake::{
        Epoch, ProcessId, Snowflake, SnowflakeGenerator, SnowflakeIncrement, SnowflakeTimestamp,
        SnowflakeTimestampFromDateTimeError, WorkerId,
    };
    use time::{Duration, UtcDateTime, macros::utc_datetime};

    struct MillennialEpoch;
    impl Epoch for MillennialEpoch {
        const EPOCH_TIME: UtcDateTime = utc_datetime!(2000-1-1 00:00);
    }

    #[test]
    fn legal_values() {
        let legal_timestamps = [0, 0xFFFF, 0x03FF_FFFF_FFFF];
        let illegal_timestamps = [0x0400_0000_0000, 0x08F0_0000_0000_0000, u64::MAX];

        for legal_timestamp in legal_timestamps {
            assert!(SnowflakeTimestamp::<MillennialEpoch>::new(legal_timestamp).is_some());
        }
        for illegal_timestamp in illegal_timestamps {
            assert!(SnowflakeTimestamp::<MillennialEpoch>::new(illegal_timestamp).is_none());
        }

        let legal_ids = [0, 0xD, 0x1F];
        let illegal_ids = [0x20, 0xF0, u8::MAX];

        for legal_id in legal_ids {
            assert!(WorkerId::new(legal_id).is_some());
            assert!(ProcessId::new(legal_id).is_some());
        }
        for illegal_id in illegal_ids {
            assert!(WorkerId::new(illegal_id).is_none());
            assert!(ProcessId::new(illegal_id).is_none());
        }

        let legal_increments = [0, 0xFF, 0xFFF];
        let illegal_increments = [0x1000, 0xFF00, u16::MAX];

        for legal_increment in legal_increments {
            assert!(SnowflakeIncrement::new(legal_increment).is_some());
            assert!(SnowflakeIncrement::new(legal_increment).is_some());
        }
        for illegal_increment in illegal_increments {
            assert!(SnowflakeIncrement::new(illegal_increment).is_none());
            assert!(SnowflakeIncrement::new(illegal_increment).is_none());
        }
    }

    #[test]
    fn snowflake_timestamp() {
        let legal_date_times = [
            MillennialEpoch::EPOCH_TIME,
            utc_datetime!(2025-10-24 10:00),
            MillennialEpoch::EPOCH_TIME + Duration::milliseconds(0x03FF_FFFF_FFFF),
        ];

        for legal_date_time in legal_date_times {
            let timestamp =
                SnowflakeTimestamp::<MillennialEpoch>::try_from(legal_date_time).unwrap();
            assert_eq!(UtcDateTime::from(timestamp), legal_date_time);
        }

        assert_eq!(
            SnowflakeTimestamp::<MillennialEpoch>::try_from(
                MillennialEpoch::EPOCH_TIME - Duration::milliseconds(1)
            ),
            Err(SnowflakeTimestampFromDateTimeError::TimeBeforeEpoch)
        );

        assert_eq!(
            SnowflakeTimestamp::<MillennialEpoch>::try_from(
                MillennialEpoch::EPOCH_TIME + Duration::milliseconds(0x0400_0000_0000)
            ),
            Err(SnowflakeTimestampFromDateTimeError::TimestampTooLarge)
        );
    }

    #[test]
    fn snowflake_increment() {
        assert_eq!(
            SnowflakeIncrement::new_unchecked(0).next(),
            SnowflakeIncrement::new_unchecked(1)
        );
        assert_eq!(
            SnowflakeIncrement::new_unchecked(100).next(),
            SnowflakeIncrement::new_unchecked(101)
        );
        assert_eq!(
            SnowflakeIncrement::new_unchecked(0xFFF).next(),
            SnowflakeIncrement::new_unchecked(0)
        );

        let mut snowflake_increment = SnowflakeIncrement::new_unchecked(0xFFE);
        snowflake_increment.increment();
        assert_eq!(
            snowflake_increment,
            SnowflakeIncrement::new_unchecked(0xFFF)
        );
        snowflake_increment.increment();
        assert_eq!(snowflake_increment, SnowflakeIncrement::new_unchecked(0));
    }

    #[test]
    fn snowflake_from_into_parts() {
        let timestamp = SnowflakeTimestamp::from_time_unchecked(utc_datetime!(2025-10-24 10:30));
        let worker_id = WorkerId::new_unchecked(0b10101);
        let process_id = ProcessId::new_unchecked(0b10001);
        let increment = SnowflakeIncrement::new_unchecked(100);

        let snowflake =
            Snowflake::<MillennialEpoch>::from_parts(timestamp, worker_id, process_id, increment);

        assert_eq!(snowflake.get(), 3_416_751_341_570_822_244);

        assert_eq!(snowflake.timestamp(), timestamp);
        assert_eq!(snowflake.worker_id(), worker_id);
        assert_eq!(snowflake.process_id(), process_id);
        assert_eq!(snowflake.increment(), increment);
    }

    #[test]
    fn snowflake_generator() {
        let worker_id = WorkerId::new_unchecked(10);
        let process_id = ProcessId::new_unchecked(0);
        let time = utc_datetime!(2025-10-24 10:55);

        let mut generator = SnowflakeGenerator::<MillennialEpoch>::new(worker_id, process_id);

        let first_snowflake = generator.generate_at(time);
        assert_eq!(
            first_snowflake,
            Snowflake::from_parts(
                SnowflakeTimestamp::from_time_unchecked(time),
                worker_id,
                process_id,
                SnowflakeIncrement::new_unchecked(0)
            )
        );

        let second_snowflake = generator.generate_at(time);
        assert_eq!(
            second_snowflake,
            Snowflake::from_parts(
                SnowflakeTimestamp::from_time_unchecked(time),
                worker_id,
                process_id,
                SnowflakeIncrement::new_unchecked(1)
            )
        );
    }
}
