use oauth2::{CsrfToken, RedirectUrl};
use stellwerk_common::model::{
    ModelValidationError,
    auth::Authentication,
    oauth2::{Oauth2ProviderChoice, Oauth2State, Oauth2UserIdentityDiscord},
    post::{PartialPost, Post, PostContent},
    user::{User, UserHandle},
};
use time::{Duration, PrimitiveDateTime};

#[derive(Clone, Eq, PartialEq, Debug, Default, Hash)]
pub(crate) struct UserRecord {
    pub user_snowflake: i64,
    pub handle: String,
}

#[derive(Clone, Eq, PartialEq, Debug, Default, Hash)]
pub(crate) struct PostRecord {
    pub post_snowflake: i64,
    pub content: String,
    pub user_snowflake: i64,
    pub handle: String,
}

#[derive(Clone, Eq, PartialEq, Debug, Default, Hash)]
pub(crate) struct PartialPostRecord {
    pub user_snowflake: i64,
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

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Hash, sqlx::Type)]
#[sqlx(type_name = "auth.oauth2_provider")]
pub(crate) enum Oauth2ProviderChoiceRecord {
    Discord,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub(crate) struct Oauth2StateRecord {
    pub session_id: String,
    pub auth_provider: Oauth2ProviderChoiceRecord,
    pub csrf_token: String,
    pub redirect_url: String,
    pub expires_at: PrimitiveDateTime,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Hash)]
pub struct Oauth2UserIdentityDiscordRecord {
    pub user_snowflake: i64,
    pub oauth2_discord_id: i64,
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
            author_id: value.user_snowflake.cast_unsigned().into(),
            content: PostContent {
                content: value.content,
            },
        })
    }
}

impl TryFrom<PostRecord> for Post {
    type Error = ModelValidationError;

    fn try_from(value: PostRecord) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.post_snowflake.cast_unsigned().into(),
            author: User {
                id: value.user_snowflake.cast_unsigned().into(),
                handle: UserHandle::new(value.handle)?,
            },
            content: PostContent {
                content: value.content,
            },
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

impl TryFrom<Oauth2ProviderChoiceRecord> for Oauth2ProviderChoice {
    type Error = ModelValidationError;

    fn try_from(value: Oauth2ProviderChoiceRecord) -> Result<Self, Self::Error> {
        Ok(match value {
            Oauth2ProviderChoiceRecord::Discord => Oauth2ProviderChoice::Discord,
        })
    }
}

impl From<Oauth2ProviderChoice> for Oauth2ProviderChoiceRecord {
    fn from(value: Oauth2ProviderChoice) -> Self {
        match value {
            Oauth2ProviderChoice::Discord => Oauth2ProviderChoiceRecord::Discord,
        }
    }
}

impl TryFrom<Oauth2StateRecord> for Oauth2State {
    type Error = ModelValidationError;

    fn try_from(value: Oauth2StateRecord) -> Result<Self, Self::Error> {
        Ok(Self {
            session_id: value.session_id,
            auth_provider: value.auth_provider.try_into()?,
            csrf_token: CsrfToken::new(value.csrf_token),
            redirect_url: RedirectUrl::new(value.redirect_url)?,
            expires_at: value.expires_at.as_utc(),
        })
    }
}

impl TryFrom<Oauth2UserIdentityDiscordRecord> for Oauth2UserIdentityDiscord {
    type Error = ModelValidationError;

    fn try_from(value: Oauth2UserIdentityDiscordRecord) -> Result<Self, Self::Error> {
        Ok(Self {
            user_id: value.user_snowflake.cast_unsigned().into(),
            oauth2_discord_id: value.oauth2_discord_id.cast_unsigned(),
        })
    }
}
