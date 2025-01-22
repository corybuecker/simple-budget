use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
};
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    SignedCookieJar,
};
use openidconnect::RedirectUrl;
use openidconnect::{
    core::{CoreAuthenticationFlow, CoreClient, CoreProviderMetadata},
    ClientId, ClientSecret, CsrfToken, IssuerUrl, Nonce, Scope,
};
use std::env;
use tera::Context;

use crate::{errors::FormError, SharedState};

pub async fn login(state: State<SharedState>) -> Result<Response, FormError> {
    let tera = &state.tera;
    let content = tera.render("authentication/login.html", &Context::new())?;

    Ok(Html::from(content).into_response())
}

pub async fn redirect(jar: SignedCookieJar) -> Result<(SignedCookieJar, Response), StatusCode> {
    let client_id = env::var("GOOGLE_CLIENT_ID").unwrap();
    let client_secret = env::var("GOOGLE_CLIENT_SECRET").unwrap();
    let callback_url = env::var("GOOGLE_CALLBACK_URL").unwrap();

    let Ok(issuer_url) = IssuerUrl::new("https://accounts.google.com".to_string()) else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };

    let async_http_client = openidconnect::reqwest::Client::builder().build().unwrap();

    let Ok(provider_metadata) =
        CoreProviderMetadata::discover_async(issuer_url, &async_http_client).await
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

    let secure = env::var("SECURE").unwrap_or("false".to_string());
    let cookie = Cookie::build(("nonce", nonce.secret().clone()))
        .expires(None)
        .http_only(true)
        .path("/authentication")
        .same_site(SameSite::Lax)
        .secure(secure == *"true")
        .build();

    Ok((
        jar.add(cookie),
        Redirect::to(auth_url.as_str()).into_response(),
    ))
}
