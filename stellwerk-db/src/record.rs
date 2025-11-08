use stellwerk_common::model::{
    ModelValidationError,
    auth::Authentication,
    post::{PartialPost, Post},
    user::{User, UserHandle},
};
use time::{Duration, PrimitiveDateTime};

#[derive(Clone, Eq, PartialEq, Debug, Default, Hash)]
pub(crate) struct UserRecord {
    pub user_snowflake: i64,
    pub handle: String,
}

#[derive(Clone, Eq, PartialEq, Debug, Default, Hash)]
pub(crate) struct FullPostRecord {
    pub post_snowflake: i64,
    pub content: String,
    pub user_snowflake: i64,
    pub handle: String,
}

#[derive(Clone, Eq, PartialEq, Debug, Default, Hash)]
pub(crate) struct PartialPostRecord {
    pub post_snowflake: i64,
    pub content: String,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub(crate) struct AuthenticationRecord {
    pub user_snowflake: i64,
    pub token_hash: Box<[u8]>,
    pub created_at: PrimitiveDateTime,
    pub expires_after_seconds: Option<i64>,
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

impl TryFrom<PartialPostRecord> for PartialPost {
    type Error = ModelValidationError;

    fn try_from(value: PartialPostRecord) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.post_snowflake.cast_unsigned().into(),
            content: value.content,
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

impl TryFrom<AuthenticationRecord> for Authentication {
    type Error = ModelValidationError;

    fn try_from(value: AuthenticationRecord) -> Result<Self, Self::Error> {
        Ok(Self {
            user: value.user_snowflake.cast_unsigned().into(),
            token_hash: value.token_hash.try_into()?,
            created_at: value.created_at.as_utc(),
            expires_after: value
                .expires_after_seconds
                .map(|seconds| Duration::seconds(seconds).try_into())
                .transpose()?,
        })
    }
}
