use socialmediathingnametbd_common::model::post::Post;
use socialmediathingnametbd_common::model::user::{User, UserHandle};
use thiserror::Error;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Default, Hash, Error)]
#[error("Database had invalid entry")]
pub struct DbDataError;

#[derive(Clone, Eq, PartialEq, Debug, Default, Hash)]
pub struct UserRecord {
    pub user_snowflake: i64,
    pub handle: String,
}

#[derive(Clone, Eq, PartialEq, Debug, Default, Hash)]
pub struct FullPostRecord {
    pub post_snowflake: i64,
    pub content: String,
    pub user_snowflake: i64,
    pub handle: String,
}

impl TryFrom<UserRecord> for User {
    type Error = DbDataError;

    fn try_from(value: UserRecord) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.user_snowflake.cast_unsigned().into(),
            handle: UserHandle::new(value.handle).ok_or(DbDataError)?,
        })
    }
}

impl TryFrom<FullPostRecord> for Post {
    type Error = DbDataError;

    fn try_from(value: FullPostRecord) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.post_snowflake.cast_unsigned().into(),
            author: User {
                id: value.user_snowflake.cast_unsigned().into(),
                handle: UserHandle::new(value.handle).ok_or(DbDataError)?,
            },
            content: value.content,
        })
    }
}
