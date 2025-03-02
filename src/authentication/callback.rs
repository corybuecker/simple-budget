use crate::{
    SharedState,
    errors::FormError,
    models::user::{Session, User},
};
use anyhow::{Result, anyhow};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::{
    Query, SignedCookieJar,
    cookie::{Cookie, SameSite},
};
use chrono::{Days, Utc};
use openidconnect::RedirectUrl;
use openidconnect::{
    AuthorizationCode, ClientId, ClientSecret, IssuerUrl, Nonce, TokenResponse,
    core::{CoreClient, CoreProviderMetadata},
};
use rand::{
    distr::{Alphanumeric, SampleString},
    rng,
};
use serde::Deserialize;
use std::env;
use tokio_postgres::Client;
use tracing::debug;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct GoogleCallback {
    code: String,
}

pub async fn callback(
    shared_state: State<SharedState>,
    query: Query<GoogleCallback>,
    jar: SignedCookieJar,
) -> Result<(SignedCookieJar, Response), FormError> {
    let client_id = env::var("GOOGLE_CLIENT_ID").unwrap();
    let client_secret = env::var("GOOGLE_CLIENT_SECRET").unwrap();
    let callback_url = env::var("GOOGLE_CALLBACK_URL").unwrap();

    let Ok(issuer_url) = IssuerUrl::new("https://accounts.google.com".to_string()) else {
        return Err(FormError {
            message: String::new(),
            status_code: Some(StatusCode::SERVICE_UNAVAILABLE),
        });
    };

    let async_http_client = openidconnect::reqwest::Client::builder().build().unwrap();

    let Ok(provider_metadata) =
        CoreProviderMetadata::discover_async(issuer_url, &async_http_client).await
    else {
        return Err(FormError {
            message: String::new(),
            status_code: Some(StatusCode::SERVICE_UNAVAILABLE),
        });
    };

    let Ok(redirect_uri) = RedirectUrl::new(callback_url) else {
        return Err(FormError {
            message: String::new(),
            status_code: Some(StatusCode::SERVICE_UNAVAILABLE),
        });
    };

    let Some(nonce_cookie) = jar.get("nonce") else {
        return Err(FormError {
            message: String::new(),
            status_code: Some(StatusCode::FORBIDDEN),
        });
    };

    let redirect_cookie = jar.get("redirect_to");
    let redirect = match &redirect_cookie {
        Some(cookie) => cookie.value(),
        None => "/",
    };

    let jar = jar.remove(Cookie::from("redirect_to"));

    let client = CoreClient::from_provider_metadata(
        provider_metadata,
        ClientId::new(client_id),
        Some(ClientSecret::new(client_secret)),
    );

    let client = client.set_redirect_uri(redirect_uri);
    let nonce = Nonce::new(nonce_cookie.value().to_string());

    let Ok(token_response) = client
        .exchange_code(AuthorizationCode::new(query.code.to_string()))
        .unwrap()
        .request_async(&async_http_client)
        .await
    else {
        return Err(FormError {
            message: String::new(),
            status_code: Some(StatusCode::SERVICE_UNAVAILABLE),
        });
    };

    let id_token = token_response.id_token().unwrap();

    let Ok(claims) = id_token.claims(&client.id_token_verifier(), &nonce) else {
        return Err(FormError {
            message: String::new(),
            status_code: Some(StatusCode::FORBIDDEN),
        });
    };

    let subject = claims.subject().to_string();
    let Some(email) = claims.email() else {
        return Err(FormError {
            message: String::new(),
            status_code: Some(StatusCode::FORBIDDEN),
        });
    };
    let email = email.to_string();
    let secure = env::var("SECURE").unwrap_or("false".to_string());

    match create_session(&shared_state.client, &subject, &email).await {
        Ok(id) => {
            let cookie = Cookie::build(("session_id", id.to_string()))
                .expires(None)
                .http_only(true)
                .path("/")
                .same_site(SameSite::Lax)
                .secure(secure == *"true")
                .build();

            Ok((jar.add(cookie), Redirect::to(redirect).into_response()))
        }
        Err(_code) => Err(FormError {
            message: String::new(),
            status_code: Some(StatusCode::FORBIDDEN),
        }),
    }
}

async fn create_session(client: &Client, subject: &str, email: &str) -> Result<Uuid> {
    let csrf = Alphanumeric.sample_string(&mut rng(), 32);

    let user = upsert_subject(client, subject.to_owned(), email.to_owned()).await?;

    let expiration = Utc::now().checked_add_days(Days::new(1)).expect("msg");
    let mut session = Session {
        id: None,
        user_id: user.id,
        expiration,
        csrf: csrf.clone(),
    };

    session.create(client).await?;
    let id = session.id.to_owned();

    id.ok_or(anyhow!("could not create a session"))
}

async fn upsert_subject(client: &Client, subject: String, email: String) -> Result<User> {
    match User::get_by_subject(client, subject.clone()).await {
        Ok(user) => Ok(user),
        Err(e) => {
            debug!("🚧 {:#?}", e);
            Ok(User::create(client, email, subject).await?)
        }
    }
}
