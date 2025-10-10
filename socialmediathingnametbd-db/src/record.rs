use socialmediathingnametbd_common::model::ModelValidationError;
use socialmediathingnametbd_common::model::post::Post;
use socialmediathingnametbd_common::model::user::{User, UserHandle};

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
    type Error = ModelValidationError;

    fn try_from(value: UserRecord) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.user_snowflake.cast_unsigned().into(),
            handle: UserHandle::new(value.handle)?,
        })
    }
}

impl TryFrom<FullPostRecord> for Post {
    type Error = ModelValidationError;

    fn try_from(value: FullPostRecord) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.post_snowflake.cast_unsigned().into(),
            author: User {
                id: value.user_snowflake.cast_unsigned().into(),
                handle: UserHandle::new(value.handle)?,
            },
            content: value.content,
        })
    }
}
