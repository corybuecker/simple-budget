use axum::{
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    SignedCookieJar,
};
use openidconnect::{
    core::{CoreAuthenticationFlow, CoreClient, CoreProviderMetadata},
    ClientId, ClientSecret, CsrfToken, IssuerUrl, Nonce, Scope,
};
use openidconnect::{reqwest::async_http_client, RedirectUrl};
use std::env;

pub async fn login(jar: SignedCookieJar) -> Result<(SignedCookieJar, Response), StatusCode> {
    let client_id = env::var("GOOGLE_CLIENT_ID").unwrap();
    let client_secret = env::var("GOOGLE_CLIENT_SECRET").unwrap();
    let callback_url = env::var("GOOGLE_CALLBACK_URL").unwrap();

    let Ok(issuer_url) = IssuerUrl::new("https://accounts.google.com".to_string()) else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };

    let Ok(provider_metadata) =
        CoreProviderMetadata::discover_async(issuer_url, async_http_client).await
    else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };

    let Ok(redirect_uri) = RedirectUrl::new(callback_url) else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };

    let client = CoreClient::from_provider_metadata(
        provider_metadata,
        ClientId::new(client_id),
        Some(ClientSecret::new(client_secret)),
    );

    let client = client.set_redirect_uri(redirect_uri);

    let (auth_url, _, nonce) = client
        .authorize_url(
            CoreAuthenticationFlow::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("openid".to_string()))
        .url();

    let cookie = Cookie::build(("nonce", nonce.secret().clone()))
        .expires(None)
        .http_only(true)
        .path("/authentication")
        .same_site(SameSite::Strict)
        .build();

    return Ok((
        jar.add(cookie),
        Redirect::to(auth_url.as_str()).into_response(),
    ));
}
