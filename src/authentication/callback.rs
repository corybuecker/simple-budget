use crate::{
    errors::FormError,
    models::user::{Preferences, Session, User},
    SharedState,
};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    Query, SignedCookieJar,
};
use chrono::{Days, Utc};
use mongodb::{
    bson::{doc, oid::ObjectId, Uuid},
    Client, Collection,
};
use openidconnect::RedirectUrl;
use openidconnect::{
    core::{CoreClient, CoreProviderMetadata},
    AuthorizationCode, ClientId, ClientSecret, IssuerUrl, Nonce, TokenResponse,
};
use rand::{
    distributions::{Alphanumeric, DistString},
    thread_rng,
};
use serde::Deserialize;
use std::env;

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

    match create_session(shared_state.mongo.clone(), subject, email).await {
        Ok(id) => {
            let cookie = Cookie::build(("session_id", id))
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

async fn create_session(
    mongo: Client,
    subject: String,
    email: String,
) -> Result<String, mongodb::error::Error> {
    let user_collection: Collection<User> = mongo.default_database().unwrap().collection("users");
    let csrf = Alphanumeric.sample_string(&mut thread_rng(), 32);

    let user = upsert_subject(mongo, subject, email).await?;

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
