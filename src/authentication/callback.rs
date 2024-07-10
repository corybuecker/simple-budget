use crate::SharedState;
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    Query, SignedCookieJar,
};
use chrono::{DateTime, Days, Utc};
use log::error;
use mongodb::{
    bson::{self, doc, oid::ObjectId, Uuid},
    Client, Collection,
};
use openidconnect::{
    core::{CoreClient, CoreProviderMetadata},
    AuthorizationCode, ClientId, ClientSecret, IssuerUrl, Nonce, TokenResponse,
};
use openidconnect::{reqwest::async_http_client, RedirectUrl};
use rand::{
    distributions::{Alphanumeric, DistString},
    thread_rng,
};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Deserialize)]
pub struct GoogleCallback {
    code: String,
}

#[derive(Deserialize, Serialize)]
struct Session {
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    expiration: DateTime<Utc>,
    id: bson::Uuid,
    csrf: String,
}

#[derive(Deserialize, Serialize)]
struct User {
    subject: String,
    email: String,
    sessions: Vec<Session>,
    _id: ObjectId,
}
pub async fn callback(
    shared_state: State<SharedState>,
    query: Query<GoogleCallback>,
    jar: SignedCookieJar,
) -> Result<(SignedCookieJar, Response), StatusCode> {
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

    let Some(nonce_cookie) = jar.get("nonce") else {
        return Err(StatusCode::FORBIDDEN);
    };

    let client = CoreClient::from_provider_metadata(
        provider_metadata,
        ClientId::new(client_id),
        Some(ClientSecret::new(client_secret)),
    );

    let client = client.set_redirect_uri(redirect_uri);
    let nonce = Nonce::new(nonce_cookie.value().to_string());

    let Ok(token_response) = client
        .exchange_code(AuthorizationCode::new(query.code.to_string()))
        .request_async(async_http_client)
        .await
    else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };

    let id_token = token_response.id_token().unwrap();

    let Ok(claims) = id_token.claims(&client.id_token_verifier(), &nonce) else {
        error!("issue with claims");
        return Err(StatusCode::FORBIDDEN);
    };

    let subject = claims.subject().to_string();
    let Some(email) = claims.email() else {
        error!("issue with email");
        return Err(StatusCode::FORBIDDEN);
    };
    let email = email.to_string();
    let secure = env::var("SECURE")
        .or::<String>(Ok("false".to_string()))
        .unwrap();

    match create_session(shared_state.mongo.clone(), subject, email).await {
        Ok(id) => {
            let cookie = Cookie::build(("session_id", id))
                .expires(None)
                .http_only(true)
                .path("/")
                .same_site(SameSite::Strict)
                .secure(secure == "true".to_string())
                .build();

            return Ok((jar.add(cookie), Html::from("OK").into_response()));
        }
        Err(code) => {
            error!("{}", code);
            return Err(StatusCode::FORBIDDEN);
        }
    }
}

async fn create_session(
    mongo: Client,
    subject: String,
    email: String,
) -> Result<String, mongodb::error::Error> {
    let user_collection: Collection<User> = mongo.database("simple_budget").collection("users");
    let csrf = Alphanumeric.sample_string(&mut thread_rng(), 32);

    let user = upsert_subject(mongo, subject, email).await?;

    let expiration = Utc::now().checked_add_days(Days::new(1)).expect("msg");
    let session = Session {
        id: Uuid::new(),
        expiration,
        csrf: csrf.clone(),
    };
    let _result = user_collection
        .update_one(
            doc! {"subject": user.subject},
            doc! {"$push": doc! {"sessions": doc! {"expiration": session.expiration, "id": session.id, "_id": ObjectId::new(), "csrf": session.csrf}}}
        )
        .await?;

    return Ok(session.id.to_string());
}

async fn upsert_subject(
    mongo: Client,
    subject: String,
    email: String,
) -> Result<User, mongodb::error::Error> {
    let user_collection: Collection<User> = mongo.database("simple_budget").collection("users");
    let existing_user = user_collection.find_one(doc! {"subject": &subject}).await;

    if existing_user.is_err() {
        return Err(existing_user.err().unwrap());
    } else {
        match existing_user.unwrap() {
            Some(user) => {
                let update = user_collection
                    .update_one(
                        doc! {"subject": &subject},
                        doc! {"$set": doc! {"email": email}},
                    )
                    .await;

                if update.is_err() {
                    let error = update.err();
                    log::info!("{:?}", &error);
                    return Err(error.unwrap());
                }

                return Ok(user);
            }
            None => {
                let user = User {
                    subject,
                    email,
                    sessions: Vec::new(),
                    _id: ObjectId::new(),
                };
                let result = user_collection.insert_one(&user).await;
                log::info!("{:?}", result);
                return Ok(user);
            }
        }
    }
}
