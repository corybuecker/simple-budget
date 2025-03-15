use anyhow::{Result, anyhow};
use axum::{
    Json,
    extract::State,
    response::{Html, IntoResponse},
};
use axum_extra::extract::{SignedCookieJar, cookie::Cookie};
use chrono::{Days, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, jwk::Jwk};
use openidconnect::{IssuerUrl, core::CoreProviderMetadata};
use rand::{
    distr::{Alphanumeric, SampleString},
    rng,
};
use serde::Deserialize;
use serde_json::json;
use std::env;
use tokio_postgres::Client;
use tracing::debug;
use uuid::Uuid;

use crate::{
    SharedState,
    errors::AppResponse,
    models::user::{Session, User},
};

#[derive(Debug, Deserialize)]
pub struct Payload {
    id_token: String,
}

#[derive(Debug, Deserialize)]
struct Claims {
    sub: String,
    email: String,
}

pub async fn token(
    shared_state: State<SharedState>,
    jar: SignedCookieJar,
    Json(token): Json<Payload>,
) -> AppResponse {
    debug!("{:#?}", token);

    let issuer_url = IssuerUrl::new("https://accounts.google.com".to_string()).unwrap();

    let async_http_client = openidconnect::reqwest::Client::builder().build().unwrap();

    let provider_metadata = CoreProviderMetadata::discover_async(issuer_url, &async_http_client)
        .await
        .unwrap();

    let keys = provider_metadata.jwks().keys();
    let key = json!(keys[0].clone());
    let jwk: Jwk = serde_json::from_value(key).unwrap();

    let mut validation = Validation::new(Algorithm::RS256);
    let aud = env::var("IOS_CLIENT_ID").unwrap();

    validation.set_audience(&[&aud]);
    validation.set_issuer(&["accounts.google.com", "https://accounts.google.com"]);

    let status = decode::<Claims>(
        &token.id_token,
        &DecodingKey::from_jwk(&jwk).unwrap(),
        &validation,
    )
    .unwrap();

    let id = create_session(
        &shared_state.client,
        &status.claims.sub,
        &status.claims.email,
    )
    .await
    .unwrap();

    let secure = env::var("SECURE").unwrap_or("false".to_string());

    let cookie = Cookie::build(("session_id", id.to_string()))
        .expires(None)
        .http_only(true)
        .path("/")
        .same_site(axum_extra::extract::cookie::SameSite::Lax)
        .secure(secure == *"true")
        .build();

    Ok((jar.add(cookie), Html::from("OK")).into_response())
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
        Err(_) => Ok(User::create(client, email, subject).await?),
    }
}
