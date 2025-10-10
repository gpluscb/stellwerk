use crate::model::Id;

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

impl UserHandle {
    #[must_use]
    pub fn new(handle: String) -> Option<Self> {
        (handle.chars().count() <= USER_HANDLE_MAX_LEN).then_some(Self(handle))
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
