use thiserror::Error;
use time::Duration;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Default, Hash)]
pub struct PositiveDuration(Duration);

impl PositiveDuration {
    #[must_use]
    pub fn new(duration: Duration) -> Option<Self> {
        duration.is_positive().then_some(Self(duration))
    }

    #[must_use]
    pub fn new_unchecked(duration: Duration) -> Self {
        Self::new(duration).expect("Duration was not positive.")
    }

    #[must_use]
    pub fn get(&self) -> Duration {
        self.0
    }
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Default, Hash, Error)]
#[error("The duration is not positive: {0}")]
pub struct NonPositiveDurationError(Duration);

impl TryFrom<Duration> for PositiveDuration {
    type Error = NonPositiveDurationError;

    fn try_from(value: Duration) -> Result<Self, Self::Error> {
        Self::new(value).ok_or(NonPositiveDurationError(value))
    }
}
