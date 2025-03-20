use super::client::get_claims_from_authorization_code;
use crate::{
    SharedState,
    errors::AppError,
    models::user::{Session, User},
};
use anyhow::{Result, anyhow};
use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::{
    Query, SignedCookieJar,
    cookie::{Cookie, SameSite},
};
use chrono::{Days, Utc};
use rand::{
    distr::{Alphanumeric, SampleString},
    rng,
};
use serde::Deserialize;
use std::env;
use tokio_postgres::{Client, GenericClient};
use tracing::error;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct GoogleCallback {
    code: String,
}

pub async fn callback(
    shared_state: State<SharedState>,
    query: Query<GoogleCallback>,
    jar: SignedCookieJar,
) -> Result<(SignedCookieJar, Response), AppError> {
    let nonce_cookie = jar
        .get("nonce")
        .ok_or(anyhow!("could not get nonce from cookie"))?;
    let redirect_cookie = jar.get("redirect_to");
    let redirect = match &redirect_cookie {
        Some(cookie) => cookie.value(),
        None => "/",
    };
    let jar = jar.remove(Cookie::from("redirect_to"));
    let nonce = nonce_cookie.value().to_string();

    let claims = get_claims_from_authorization_code(query.code.clone(), nonce).await?;
    let subject = claims.subject().to_string();
    let email = claims.email().ok_or(anyhow!("could not get email"))?;
    let email = email.to_string();
    let secure = env::var("SECURE").unwrap_or("false".to_string()) == "true";

    let id = create_session(shared_state.pool.get().await?.client(), &subject, &email).await?;
    let cookie = Cookie::build(("session_id", id.to_string()))
        .expires(None)
        .http_only(true)
        .path("/")
        .same_site(SameSite::Lax)
        .secure(secure)
        .build();

    Ok((jar.add(cookie), Redirect::to(redirect).into_response()))
}

async fn create_session(client: &Client, subject: &str, email: &str) -> Result<Uuid, AppError> {
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

    id.ok_or(anyhow!("could not create a session").into())
}

async fn upsert_subject(client: &Client, subject: String, email: String) -> Result<User, AppError> {
    match User::get_by_subject(client, subject.clone()).await {
        Ok(user) => Ok(user),
        Err(e) => {
            error!("🚧 {:#?}", e);

            Ok(User::create(client, email, subject).await?)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::create_session;
    use crate::{models::user::User, test_utils::client_for_tests};
    use tokio_postgres::GenericClient;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_create_session_for_new_user() {
        let uuid = Uuid::new_v4().to_string();
        let client = client_for_tests().await.unwrap();
        let client = client.client();
        let session = create_session(client, &uuid, &uuid).await;
        assert!(session.is_ok());
        let user = User::get_by_subject(client, uuid.clone()).await;
        assert!(user.is_ok());
    }

    #[tokio::test]
    async fn test_create_session_for_existing_user() {
        let uuid = &Uuid::new_v4().to_string();
        let client = client_for_tests().await.unwrap();
        let client = client.client();
        let user = User::create(client, uuid.clone(), uuid.clone()).await;
        assert!(user.is_ok());
        let session = create_session(client, &uuid, &uuid).await;
        assert!(session.is_ok());
        let user = User::get_by_subject(client, uuid.to_string()).await;
        assert!(user.is_ok());
    }
}
