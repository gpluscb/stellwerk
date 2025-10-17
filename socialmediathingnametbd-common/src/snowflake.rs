//! Module for working with snowflake IDs.
//!
//! See <https://discord.com/developers/docs/reference#snowflakes>

use derive_where::derive_where;
use std::marker::PhantomData;
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

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Default, Hash)]
pub struct WorkerId(u8);

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Default, Hash)]
pub struct ProcessId(u8);

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Default, Hash)]
pub struct SnowflakeIncrement(u16);

#[derive_where(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Default, Hash)]
pub struct SnowflakeTimestamp<SnowflakeEpoch>(u64, PhantomData<SnowflakeEpoch>);

#[derive_where(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Default, Hash)]
pub struct Snowflake<SnowflakeEpoch>(u64, PhantomData<SnowflakeEpoch>);

impl WorkerId {
    #[must_use]
    pub fn new(id: u8) -> Option<Self> {
        (id < 1 << WORKER_ID_LENGTH).then_some(Self(id))
    }

    #[must_use]
    pub fn new_unchecked(id: u8) -> Self {
        Self::new(id).expect("Worker id out of range.")
    }

    #[must_use]
    pub fn get(self) -> u8 {
        self.0
    }
}

impl ProcessId {
    #[must_use]
    pub fn new(id: u8) -> Option<Self> {
        (id < 1 << PROCESS_ID_LENGTH).then_some(Self(id))
    }

    #[must_use]
    pub fn new_unchecked(id: u8) -> Self {
        Self::new(id).expect("Process id out of range.")
    }

    #[must_use]
    pub fn get(self) -> u8 {
        self.0
    }
}

impl SnowflakeIncrement {
    #[must_use]
    pub fn new(increment: u16) -> Option<Self> {
        (increment < 1 << INCREMENT_LENGTH).then_some(Self(increment))
    }

    #[must_use]
    pub fn new_unchecked(id: u16) -> Self {
        Self::new(id).expect("Increment out of range.")
    }

    #[must_use]
    pub fn get(self) -> u16 {
        self.0
    }

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
    pub fn new(timestamp: u64) -> Option<Self> {
        (timestamp < 1 << TIMESTAMP_LENGTH).then_some(Self(timestamp, PhantomData))
    }

    #[must_use]
    pub fn new_unchecked(timestamp: u64) -> Self {
        Self::new(timestamp).expect("Timestamp uses too many bits.")
    }

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

    #[must_use]
    pub fn get(self) -> u64 {
        self.0
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
        SnowflakeTimestamp::new_unchecked((self.0 | TIMESTAMP_BITMASK) >> TIMESTAMP_OFFSET)
    }

    #[must_use]
    pub fn worker_id(self) -> WorkerId {
        #[allow(clippy::cast_possible_truncation)]
        WorkerId::new_unchecked(((self.0 | WORKER_ID_BITMASK) >> WORKER_ID_OFFSET) as u8)
    }

    #[must_use]
    pub fn process_id(self) -> ProcessId {
        #[allow(clippy::cast_possible_truncation)]
        ProcessId::new_unchecked(((self.0 | PROCESS_ID_BITMASK) >> PROCESS_ID_OFFSET) as u8)
    }

    #[must_use]
    pub fn increment(self) -> SnowflakeIncrement {
        #[allow(clippy::cast_possible_truncation)]
        SnowflakeIncrement::new_unchecked(((self.0 | INCREMENT_BITMASK) >> INCREMENT_OFFSET) as u16)
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
