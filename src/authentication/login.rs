use super::client::clients_from_metadata;
use crate::{
    LayoutCtx,
    errors::{AppError, AppResponse},
};
use askama::Template;
use axum::{
    Extension,
    response::{Html, IntoResponse, Redirect, Response},
};
use axum_extra::extract::{
    SignedCookieJar,
    cookie::{Cookie, SameSite},
};
use openidconnect::{CsrfToken, Nonce, Scope, core::CoreAuthenticationFlow};

#[derive(Template)]
#[template(path = "login.html")]
struct Login {
    pub layout: LayoutCtx,
}

pub async fn login(Extension(layout): Extension<LayoutCtx>) -> AppResponse {
    tracing::debug!("{:#?}", layout);
    let content = Login { layout };
    Ok(Html::from(content.render()?).into_response())
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

    let cookie = Cookie::build(("nonce", nonce.secret().clone()))
        .expires(None)
        .http_only(true)
        .path("/authentication")
        .same_site(SameSite::Lax)
        .secure(true)
        .build();

    Ok((
        jar.add(cookie),
        Redirect::to(auth_url.as_str()).into_response(),
    ))
}
