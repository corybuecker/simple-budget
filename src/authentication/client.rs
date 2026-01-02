use anyhow::{Result, anyhow};
use openidconnect::{
    AuthorizationCode, ClientId, ClientSecret, EmptyAdditionalClaims, EndpointMaybeSet,
    EndpointNotSet, EndpointSet, IdTokenClaims, IssuerUrl, Nonce, RedirectUrl,
    RevocationErrorResponseType, StandardErrorResponse, TokenResponse,
    core::{
        CoreAuthDisplay, CoreAuthPrompt, CoreClient, CoreErrorResponseType, CoreGenderClaim,
        CoreJsonWebKey, CoreJweContentEncryptionAlgorithm, CoreProviderMetadata,
        CoreRevocableToken, CoreTokenIntrospectionResponse, CoreTokenResponse,
    },
};
use std::env;

type OidcClient = openidconnect::Client<
    EmptyAdditionalClaims,
    CoreAuthDisplay,
    CoreGenderClaim,
    CoreJweContentEncryptionAlgorithm,
    CoreJsonWebKey,
    CoreAuthPrompt,
    StandardErrorResponse<CoreErrorResponseType>,
    CoreTokenResponse,
    CoreTokenIntrospectionResponse,
    CoreRevocableToken,
    StandardErrorResponse<RevocationErrorResponseType>,
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointMaybeSet,
    EndpointMaybeSet,
>;

pub async fn clients_from_metadata() -> Result<(OidcClient, openidconnect::reqwest::Client)> {
    let client_id = env::var("GOOGLE_CLIENT_ID")?;
    let client_secret = env::var("GOOGLE_CLIENT_SECRET")?;
    let callback_url = env::var("GOOGLE_CALLBACK_URL")?;
    let issuer_url = IssuerUrl::new("https://accounts.google.com".to_string())?;
    let async_http_client = openidconnect::reqwest::Client::builder().build()?;
    let provider_metadata =
        CoreProviderMetadata::discover_async(issuer_url, &async_http_client).await?;
    let redirect_uri = RedirectUrl::new(callback_url)?;
    let client = CoreClient::from_provider_metadata(
        provider_metadata,
        ClientId::new(client_id),
        Some(ClientSecret::new(client_secret)),
    );
    let client = client.set_redirect_uri(redirect_uri);

    Ok((client, async_http_client))
}

pub async fn get_claims_from_authorization_code(
    code: String,
    nonce: String,
) -> Result<IdTokenClaims<EmptyAdditionalClaims, CoreGenderClaim>> {
    let (oidc_client, http_client) = clients_from_metadata().await?;
    let token_response = oidc_client
        .exchange_code(AuthorizationCode::new(code))?
        .request_async(&http_client)
        .await?;
    let id_token = token_response
        .id_token()
        .ok_or(anyhow!("could not get id token"))?;
    let nonce = Nonce::new(nonce);
    let claims = id_token.claims(&oidc_client.id_token_verifier(), &nonce)?;

    Ok(claims.clone())
}
