use crate::{
    login_logout::LoginLogoutService,
    oauth2::{AuthTokenResponse, AuthUrlResponse, Oauth2Service, get_identity_from_provider},
    server::{
        Result, ServerError, ServerRouter, json::Json, query::Query, typed_path::PathWrapper,
    },
};
use axum::extract::State;
use axum_extra::routing::TypedPath;
use oauth2::{AuthorizationCode, CsrfToken, RedirectUrl, TokenResponse, url::Url};
use schemars::JsonSchema;
use serde::Deserialize;
use std::{borrow::Cow, sync::Arc};
use stellwerk_common::{
    json_schema_wrappers::JsonSchemaOffsetDateTime,
    model::oauth2::{Oauth2ProviderChoice, Oauth2State},
};
use stellwerk_db::client::DbClient;
use time::{Duration, UtcDateTime, UtcOffset};
use tracing::error;

pub fn routes() -> ServerRouter {
    ServerRouter::new()
        .typed_get(get_oauth2_url)
        .typed_get(get_token)
}

#[derive(TypedPath, Deserialize, JsonSchema)]
#[typed_path("/oauth2/auth-url", rejection(ServerError))]
struct GetAuthUrlPath {}
#[derive(Deserialize, JsonSchema)]
struct GetAuthUrlParams {
    provider: Oauth2ProviderChoice,
    redirect: Url,
    session_id: String,
}

async fn get_oauth2_url(
    PathWrapper(GetAuthUrlPath {}): PathWrapper<GetAuthUrlPath>,
    Query(params): Query<GetAuthUrlParams>,
    State(oauth2_service): State<Arc<Oauth2Service>>,
    State(db): State<Arc<DbClient>>,
) -> Result<Json<AuthUrlResponse>> {
    let oauth2_provider = oauth2_service.providers.get_provider(params.provider);
    let redirect_url = RedirectUrl::from_url(params.redirect);

    let (url, csrf_token) = oauth2_provider
        .client
        .authorize_url(CsrfToken::new_random)
        .set_redirect_uri(Cow::Borrowed(&redirect_url))
        .add_scopes(oauth2_provider.scopes.iter().cloned())
        .url();

    let oauth2_state = Oauth2State {
        session_id: params.session_id,
        auth_provider: params.provider,
        csrf_token,
        redirect_url,
        expires_at: UtcDateTime::now() + Duration::minutes(30),
    };
    db.create_oauth2_state(&oauth2_state).await?;

    Ok(Json(AuthUrlResponse { url }))
}

#[derive(TypedPath, Deserialize, JsonSchema)]
#[typed_path("/oauth2/get-token", rejection(ServerError))]
struct GetTokenPath {}
#[derive(Deserialize, JsonSchema)]
struct GetTokenParams {
    code: String,
    csrf_token: String,
    session_id: String,
    expires: bool,
}

async fn get_token(
    PathWrapper(GetTokenPath {}): PathWrapper<GetTokenPath>,
    Query(params): Query<GetTokenParams>,
    State(oauth2_service): State<Arc<Oauth2Service>>,
    State(login_logout_service): State<Arc<LoginLogoutService>>,
    State(db): State<Arc<DbClient>>,
) -> Result<Json<AuthTokenResponse>> {
    let code = AuthorizationCode::new(params.code);
    let csrf_token = CsrfToken::new(params.csrf_token);

    let stored_oauth2_state = db
        .fetch_oauth2_state(&params.session_id)
        .await?
        .ok_or_else(|| ServerError::Oauth2NoStateForSession(params.session_id.clone()))?;

    if stored_oauth2_state.expires_at < UtcDateTime::now() {
        return Err(ServerError::Oauth2NoStateForSession(params.session_id));
    }

    if stored_oauth2_state.csrf_token != csrf_token {
        return Err(ServerError::Oauth2WrongCsrfToken);
    }

    // State has been used and can be deleted
    // Since we don't use transactions, a user could use a race to use the same oauth2 state
    // multiple times to generate multiple tokens, but this is not harmful.
    db.delete_oauth2_state(&params.session_id).await?;

    let auth_provider = oauth2_service
        .providers
        .get_provider(stored_oauth2_state.auth_provider);

    // Get auth provider token
    let token_response = auth_provider
        .client
        .exchange_code(code)
        .set_redirect_uri(Cow::Owned(stored_oauth2_state.redirect_url))
        .request_async(&oauth2_service.http_client)
        .await?;
    let access_token = token_response.access_token();

    // Use auth provider token to verify identity
    let user_id_result =
        get_identity_from_provider(&db, stored_oauth2_state.auth_provider, access_token.clone())
            .await;

    // Auth provider token is useless after identity verification, try to revoke
    match auth_provider.client.revoke_token(access_token.into()) {
        Ok(revocation_request) => {
            if let Err(error) = revocation_request
                .request_async(&oauth2_service.http_client)
                .await
            {
                error!(%error, "Error executing token revocation");
            }
        }
        Err(error) => error!(%error, "Revocation request configuration error"),
    }

    // Return on errors only after revoking
    let user_id = user_id_result?.ok_or(ServerError::Oauth2NoAssociatedUser)?;

    // Log in user
    let login_data = login_logout_service
        .login_user(&db, user_id, params.expires)
        .await?;

    Ok(Json(AuthTokenResponse {
        token: login_data.token.token_str(),
        expires_at: login_data
            .expires_at
            .map(|utc_date_time| utc_date_time.to_offset(UtcOffset::UTC))
            .map(JsonSchemaOffsetDateTime),
    }))
}
