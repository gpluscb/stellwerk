use oauth2::{AuthUrl, ClientId, ClientSecret, RevocationUrl, Scope, TokenUrl};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, path::Path};
use stellwerk_common::{
    positive_duration::PositiveDuration,
    snowflake::{ProcessId, WorkerId},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReadConfigError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Toml(#[from] toml::de::Error),
}

pub fn read_config(path: impl AsRef<Path>) -> Result<ApiConfig, ReadConfigError> {
    let file = std::fs::read(path)?;
    let config = toml::from_slice(&file)?;
    Ok(config)
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Serialize, Deserialize)]
pub struct ApiConfig {
    pub server_address: SocketAddr,
    pub database_url: String,
    pub worker_id: WorkerId,
    pub process_id: ProcessId,
    pub oauth2_providers_config: ApiOauth2ProvidersConfig,
    pub login_logout_config: LoginLogoutConfig,
}

#[derive(
    Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Default, Hash, Serialize, Deserialize,
)]
pub struct LoginLogoutConfig {
    pub expiring_token_duration_seconds: PositiveDuration,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Serialize, Deserialize)]
pub struct ApiOauth2ProvidersConfig {
    pub discord: ApiOauth2ProviderConfig,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Serialize, Deserialize)]
pub struct ApiOauth2ProviderConfig {
    pub client_id: ClientId,
    pub client_secret: ClientSecret,
    pub auth_url: AuthUrl,
    pub token_url: TokenUrl,
    pub revocation_url: RevocationUrl,
    pub scopes: Vec<Scope>,
}
