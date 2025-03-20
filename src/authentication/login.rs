use super::client::clients_from_metadata;
use crate::{
    SharedState,
    errors::{AppError, AppResponse},
};
use axum::{
    extract::State,
    response::{Html, IntoResponse, Redirect, Response},
};
use axum_extra::extract::{
    SignedCookieJar,
    cookie::{Cookie, SameSite},
};
use openidconnect::{CsrfToken, Nonce, Scope, core::CoreAuthenticationFlow};
use std::env;
use tera::Context;

pub async fn login(state: State<SharedState>) -> AppResponse {
    let tera = &state.tera;
    let content = tera.render("authentication/login.html", &Context::new())?;

    Ok(Html::from(content).into_response())
}

pub async fn redirect(jar: SignedCookieJar) -> Result<(SignedCookieJar, Response), AppError> {
    let (oidc_client, _http_client) = clients_from_metadata().await?;

    let (auth_url, _, nonce) = oidc_client
        .authorize_url(
            CoreAuthenticationFlow::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("openid".to_string()))
        .url();

    let secure = env::var("SECURE").unwrap_or("false".to_string()) == "true";
    let cookie = Cookie::build(("nonce", nonce.secret().clone()))
        .expires(None)
        .http_only(true)
        .path("/authentication")
        .same_site(SameSite::Lax)
        .secure(secure)
        .build();

    Ok((
        jar.add(cookie),
        Redirect::to(auth_url.as_str()).into_response(),
    ))
}
