use crate::model::{id::Id, user::UserMarker};
use oauth2::{CsrfToken, RedirectUrl};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use time::UtcDateTime;

#[derive(
    Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Hash, Serialize, Deserialize, JsonSchema,
)]
pub enum Oauth2ProviderChoice {
    Discord,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub struct Oauth2State {
    pub session_id: String,
    pub auth_provider: Oauth2ProviderChoice,
    pub csrf_token: CsrfToken,
    pub redirect_url: RedirectUrl,
    pub expires_at: UtcDateTime,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Hash)]
pub struct Oauth2UserIdentityDiscord {
    pub user_id: Id<UserMarker>,
    pub oauth2_discord_id: u64,
}
