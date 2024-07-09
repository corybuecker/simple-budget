use crate::SharedState;
use axum::{
    extract::{Request, State},
    http::{HeaderMap, Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Redirect, Response},
    routing::get,
    Extension, Router,
};
use axum_extra::extract::SignedCookieJar;
use chrono::{DateTime, Utc};
use mongodb::{
    bson::{self, doc, oid::ObjectId, Uuid},
    options::FindOneOptions,
    Collection,
};
use serde::{Deserialize, Serialize};
mod accounts;
mod dashboard;
mod envelopes;
mod goals;

#[derive(Deserialize, Serialize, Debug)]
struct Session {
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    expiration: DateTime<Utc>,
    id: bson::Uuid,
    _id: ObjectId,
    csrf: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct User {
    _id: ObjectId,
    subject: String,
    email: String,
    sessions: Vec<Session>,
}

#[derive(Debug, Clone)]
pub struct UserExtension {
    id: String,
    csrf: String,
}

async fn validate_csrf(
    user: Extension<UserExtension>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Response {
    let method = request.method().to_owned();

    match method {
        Method::POST | Method::PUT | Method::PATCH | Method::DELETE => {
            let Some(header) = headers.get("x-csrf-token") else {
                return StatusCode::BAD_REQUEST.into_response();
            };
            if user.csrf == header.clone() {
                return next.run(request).await;
            } else {
                return StatusCode::BAD_REQUEST.into_response();
            }
        }
        _ => {
            return next.run(request).await;
        }
    }
}

async fn authenticated(
    State(state): State<SharedState>,
    jar: SignedCookieJar,
    mut request: Request,
    next: Next,
) -> Response {
    let Some(session_id) = jar.get("session_id") else {
        return Redirect::to("authentication/login").into_response();
    };

    let session_id = session_id.value();
    let users: Collection<User> = state.mongo.database("simple_budget").collection("users");
    let option = FindOneOptions::builder()
        .projection(doc! {"sessions.$": 1, "email": 1, "subject": 1})
        .build();
    let user = users
        .find_one(doc! {"sessions.id": Uuid::parse_str(session_id).unwrap()})
        .with_options(option)
        .await;

    if let Ok(Some(user)) = user {
        request.extensions_mut().insert(UserExtension {
            id: user._id.to_hex(),
            csrf: user.sessions[0].csrf.clone(),
        });
        next.run(request).await
    } else {
        Redirect::to("authentication/login").into_response()
    }
}
pub fn authenticated_router(state: SharedState) -> Router<SharedState> {
    Router::new()
        .nest("/accounts", accounts::accounts_router())
        .nest("/goals", goals::goals_router())
        .nest("/envelopes", envelopes::envelopes_router())
        .route("/reports", get(dashboard::index))
        .route_layer(middleware::from_fn(validate_csrf))
        .route_layer(middleware::from_fn_with_state(state, authenticated))
}
