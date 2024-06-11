use crate::SharedState;
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Response},
    Router,
};
use axum_extra::extract::SignedCookieJar;
use chrono::{DateTime, Utc};
use mongodb::{
    bson::{self, doc, oid::ObjectId, Uuid},
    Collection,
};
use serde::{Deserialize, Serialize};
mod accounts;
mod goals;
mod savings;

#[derive(Deserialize, Serialize, Debug)]
struct Session {
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    expiration: DateTime<Utc>,
    id: bson::Uuid,
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
}

async fn authenticated(
    State(state): State<SharedState>,
    jar: SignedCookieJar,
    mut request: Request,
    next: Next,
) -> Response {
    let Some(session_id) = jar.get("session_id") else {
        return StatusCode::FORBIDDEN.into_response();
    };

    let session_id = session_id.value();
    log::debug!("{:?}", session_id);

    let users: Collection<User> = state.mongo.database("simple_budget").collection("users");
    let user = users
        .find_one(
            doc! {"sessions.id": Uuid::parse_str(session_id).unwrap()},
            None,
        )
        .await;

    if let Ok(Some(user)) = user {
        log::debug!("{:?}", user);
        request.extensions_mut().insert(UserExtension {
            id: user._id.to_hex(),
        });
        next.run(request).await
    } else {
        StatusCode::FORBIDDEN.into_response()
    }
}
pub fn api_router(state: SharedState) -> Router<SharedState> {
    Router::new()
        .nest("/accounts", accounts::accounts_router())
        .nest("/goals", goals::goals_router())
        .nest("/savings", savings::savings_router())
        .route_layer(middleware::from_fn_with_state(state, authenticated))
}
