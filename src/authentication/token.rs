use crate::models::user::Preferences;
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Json,
};
use axum_extra::extract::{cookie::Cookie, SignedCookieJar};
use bson::{doc, oid::ObjectId, Uuid};
use chrono::{Days, Utc};
use jsonwebtoken::{decode, jwk::Jwk, Algorithm, DecodingKey, Validation};
use mongodb::{Client, Collection};
use openidconnect::{core::CoreProviderMetadata, reqwest::async_http_client, IssuerUrl};
use rand::{
    distributions::{Alphanumeric, DistString},
    thread_rng,
};
use serde::Deserialize;
use serde_json::json;
use std::env;
use tracing::debug;

use crate::{
    errors::FormError,
    models::user::{Session, User},
    SharedState,
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
) -> Result<Response, FormError> {
    debug!("{:#?}", token);

    let Ok(issuer_url) = IssuerUrl::new("https://accounts.google.com".to_string()) else {
        return Err(FormError {
            message: String::new(),
            status_code: None,
        });
    };

    let Ok(provider_metadata) =
        CoreProviderMetadata::discover_async(issuer_url, async_http_client).await
    else {
        return Err(FormError {
            message: String::new(),
            status_code: Some(StatusCode::SERVICE_UNAVAILABLE),
        });
    };

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
        shared_state.mongo.clone(),
        &status.claims.sub,
        &status.claims.email,
    )
    .await?;

    let secure = env::var("SECURE").unwrap_or("false".to_string());

    let cookie = Cookie::build(("session_id", id))
        .expires(None)
        .http_only(true)
        .path("/")
        .same_site(axum_extra::extract::cookie::SameSite::Lax)
        .secure(secure == *"true")
        .build();

    Ok((jar.add(cookie), Html::from("OK")).into_response())
}

async fn create_session(
    mongo: Client,
    subject: &str,
    email: &str,
) -> Result<String, mongodb::error::Error> {
    let user_collection: Collection<User> = mongo.default_database().unwrap().collection("users");
    let csrf = Alphanumeric.sample_string(&mut thread_rng(), 32);

    let user = upsert_subject(mongo, subject.to_owned(), email.to_owned()).await?;

    let expiration = Utc::now().checked_add_days(Days::new(1)).expect("msg");
    let session = Session {
        _id: ObjectId::new().to_string(),
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

    Ok(session.id.to_string())
}

async fn upsert_subject(
    mongo: Client,
    subject: String,
    email: String,
) -> Result<User, mongodb::error::Error> {
    let user_collection: Collection<User> = mongo.default_database().unwrap().collection("users");
    let existing_user = user_collection.find_one(doc! {"subject": &subject}).await;

    if existing_user.is_err() {
        Err(existing_user.err().unwrap())
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

                Ok(user)
            }
            None => {
                let user = User {
                    subject,
                    email,
                    preferences: Preferences {
                        timezone: None,
                        goal_header: None,
                        forecast_offset: None,
                    },
                    sessions: Some(Vec::new()),
                    _id: ObjectId::new().to_string(),
                };
                let result = user_collection.insert_one(&user).await;
                log::info!("{:?}", result);
                Ok(user)
            }
        }
    }
}
