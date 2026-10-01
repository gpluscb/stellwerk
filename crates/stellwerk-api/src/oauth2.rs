use crate::config::ApiOauth2ProvidersConfig;
use oauth2::{
    AccessToken, EndpointNotSet, EndpointSet, Scope, basic::BasicClient, reqwest,
    reqwest::redirect::Policy, url::Url,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use stellwerk_common::{
    json_schema_wrappers::JsonSchemaOffsetDateTime,
    model::{id::Id, oauth2::Oauth2ProviderChoice, user::UserMarker},
};
use stellwerk_db::client::{DbClient, DbError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Oauth2SetupError {
    #[error(transparent)]
    ReqwestConfig(#[from] reqwest::Error),
}

pub type ProviderClient =
    BasicClient<EndpointSet, EndpointNotSet, EndpointNotSet, EndpointSet, EndpointSet>;

#[derive(Clone, Debug)]
pub struct Oauth2Service {
    pub providers: Oauth2ProviderList,
    pub http_client: reqwest::Client,
}

impl Oauth2Service {
    pub fn new(config: ApiOauth2ProvidersConfig) -> Result<Self, Oauth2SetupError> {
        let config = Oauth2Service {
            providers: Oauth2ProviderList {
                discord: Oauth2Provider {
                    client: BasicClient::new(config.discord.client_id)
                        .set_client_secret(config.discord.client_secret)
                        .set_auth_uri(config.discord.auth_url)
                        .set_token_uri(config.discord.token_url)
                        .set_revocation_url(config.discord.revocation_url),
                    scopes: config.discord.scopes,
                },
            },
            http_client: reqwest::Client::builder()
                .redirect(Policy::none())
                .build()?,
        };
        Ok(config)
    }
}

#[derive(Clone, Debug)]
pub struct Oauth2Provider {
    pub client: ProviderClient,
    pub scopes: Vec<Scope>,
}

#[derive(Clone, Debug)]
pub struct Oauth2ProviderList {
    pub discord: Oauth2Provider,
}

impl Oauth2ProviderList {
    #[must_use]
    pub fn get_provider(&self, provider_choice: Oauth2ProviderChoice) -> &Oauth2Provider {
        match provider_choice {
            Oauth2ProviderChoice::Discord => &self.discord,
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Serialize, Deserialize, JsonSchema)]
pub struct AuthUrlResponse {
    pub url: Url,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Serialize, Deserialize, JsonSchema)]
pub struct AuthTokenResponse {
    pub token: String,
    // TODO: Replace with UtcDateTime if https://github.com/GREsau/schemars/pull/472/ ever lands
    pub expires_at: Option<JsonSchemaOffsetDateTime>,
}

#[derive(Debug, Error)]
pub enum Oauth2IdentityRetrievalError {
    #[error(transparent)]
    DiscordRequest(#[from] twilight_http::Error),
    #[error(transparent)]
    DiscordResponseDeserialize(#[from] twilight_http::response::DeserializeBodyError),
    #[error("Discord identify response did not contain user")]
    DiscordResponseUserNotPresent,
    #[error(transparent)]
    Database(#[from] DbError),
}

pub async fn get_identity_from_provider(
    db: &DbClient,
    oauth2provider_choice: Oauth2ProviderChoice,
    access_token: AccessToken,
) -> Result<Option<Id<UserMarker>>, Oauth2IdentityRetrievalError> {
    match oauth2provider_choice {
        Oauth2ProviderChoice::Discord => get_identity_from_discord(db, access_token).await,
    }
}

pub async fn get_identity_from_discord(
    db: &DbClient,
    access_token: AccessToken,
) -> Result<Option<Id<UserMarker>>, Oauth2IdentityRetrievalError> {
    let authorization_info =
        twilight_http::Client::new(format!("Bearer {}", access_token.into_secret()))
            .current_authorization()
            .await?
            .model()
            .await?;

    let discord_id = authorization_info
        .user
        .ok_or(Oauth2IdentityRetrievalError::DiscordResponseUserNotPresent)?
        .id;

    let identity = db.fetch_oauth2_identity_discord(discord_id.get()).await?;

    let user_id = identity.map(|identity| identity.user_id);
    Ok(user_id)
}
