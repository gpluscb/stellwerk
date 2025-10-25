use crate::{
    model::{Id, user::UserMarker},
    util::PositiveDuration,
};
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};
use time::UtcDateTime;

#[derive(Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AuthToken(pub String);

impl Debug for AuthToken {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("AuthToken").field(&"[redacted]").finish()
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub struct Authentication {
    pub user: Id<UserMarker>,
    pub token: AuthToken,
    pub created_at: UtcDateTime,
    pub expires_after: Option<PositiveDuration>,
}
