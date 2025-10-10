use crate::model::Id;
use thiserror::Error;

pub const USER_HANDLE_MAX_LEN: usize = 50;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Default, Hash)]
pub struct UserMarker;

#[derive(Clone, Eq, PartialEq, Debug, Default, Hash)]
pub struct User {
    pub id: Id<UserMarker>,
    pub handle: UserHandle,
}

#[derive(Clone, Eq, PartialEq, Debug, Default, Hash)]
pub struct CreateUser {
    pub handle: UserHandle,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Default, Hash)]
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
