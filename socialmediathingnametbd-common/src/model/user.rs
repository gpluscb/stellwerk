use crate::model::Id;
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{Error, Unexpected},
};
use thiserror::Error;

pub const USER_HANDLE_MAX_LEN: usize = 50;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Default, Hash)]
pub struct UserMarker;

#[derive(Clone, Eq, PartialEq, Debug, Default, Hash, Deserialize, Serialize)]
pub struct User {
    pub id: Id<UserMarker>,
    pub handle: UserHandle,
}

#[derive(Clone, Eq, PartialEq, Debug, Default, Hash, Deserialize, Serialize)]
pub struct CreateUser {
    pub handle: UserHandle,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Default, Hash, Serialize)]
#[serde(transparent)]
pub struct UserHandle(String);

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Default, Hash, Error)]
#[error("The user handle is invalid: {0}")]
pub struct InvalidUserHandleError(String);

impl UserHandle {
    pub fn new(handle: String) -> Result<Self, InvalidUserHandleError> {
        if handle.chars().count() <= USER_HANDLE_MAX_LEN {
            Ok(UserHandle(handle))
        } else {
            Err(InvalidUserHandleError(handle))
        }
    }

    #[must_use]
    pub fn get(&self) -> &str {
        &self.0
    }

    #[must_use]
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl<'de> Deserialize<'de> for UserHandle {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let inner = String::deserialize(deserializer)?;
        UserHandle::new(inner)
            .map_err(|err| Error::invalid_value(Unexpected::Str(&err.0), &"UserHandle"))
    }
}
