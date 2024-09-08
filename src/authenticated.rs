use crate::SharedState;
use axum::{
    extract::{ws::WebSocket, Request, State, WebSocketUpgrade},
    http::{HeaderMap, Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Redirect, Response},
    routing::get,
    Extension, Router,
};
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    SignedCookieJar,
};
use chrono::{DateTime, Utc};
use futures_util::{stream::StreamExt, SinkExt};
use mongodb::{
    bson::{self, doc, oid::ObjectId, Uuid},
    options::FindOneOptions,
    Collection,
};
use serde::{Deserialize, Serialize};
use std::env;
use tokio::{spawn, sync::watch};
use tracing::debug;
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
    pub id: String,
    pub csrf: String,
    pub channel_sender: watch::Sender<String>,

    #[allow(dead_code)]
    pub channel_receiver: watch::Receiver<String>,
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
                next.run(request).await
            } else {
                StatusCode::BAD_REQUEST.into_response()
            }
        }
        _ => next.run(request).await,
    }
}

async fn authenticated(
    State(state): State<SharedState>,
    jar: SignedCookieJar,
    mut request: Request,
    next: Next,
) -> Result<(SignedCookieJar, Response), StatusCode> {
    let Some(session_id) = jar.get("session_id") else {
        return Ok((jar, Redirect::to("authentication/login").into_response()));
    };

    let session_id = session_id.value();
    let users: Collection<User> = state.mongo.default_database().unwrap().collection("users");
    let option = FindOneOptions::builder()
        .projection(doc! {"sessions.$": 1, "email": 1, "subject": 1})
        .build();
    let user = users
        .find_one(doc! {"sessions.id": Uuid::parse_str(session_id).unwrap(), "sessions.expiration": doc! { "$gte": Utc::now() } })
        .with_options(option)
        .await;

    if let Ok(Some(user)) = user {
        let (tx, rx) = watch::channel(String::new());
        request.extensions_mut().insert(UserExtension {
            id: user._id.to_hex(),
            csrf: user.sessions[0].csrf.clone(),
            channel_sender: tx,
            channel_receiver: rx,
        });
        Ok((jar, next.run(request).await))
    } else {
        let secure = env::var("SECURE").unwrap_or("false".to_string());

        let redirect_cookie = Cookie::build(("redirect_to", request.uri().path().to_owned()))
            .expires(None)
            .http_only(true)
            .path("/authentication")
            .same_site(SameSite::Strict)
            .secure(secure == *"true")
            .build();

        Ok((
            jar.add(redirect_cookie),
            Redirect::to("authentication/login").into_response(),
        ))
    }
}

async fn message_handler(socket: WebSocket, user: UserExtension, state: SharedState) {
    let (mut sender, mut receiver) = socket.split();

    let listener = spawn(async move {
        while let Some(message) = receiver.next().await {
            if let Ok(message) = message {
                debug!("{:#?}", message);
                let _ = user.channel_sender.send("test".to_owned());
                let _ = sender.send("test received".into()).await;
                let _ = state.broker.sender.send("IN BROKER!!!".to_owned()).await;
            } else {
                return;
            };
        }
    });

    match tokio::join!(listener) {
        (Ok(_),) => {}
        (Err(join_error),) => {
            tracing::error!("{}", join_error);
        }
    }
}

async fn websocket_upgrade(
    ws: WebSocketUpgrade,
    Extension(user): Extension<UserExtension>,
    State(state): State<SharedState>,
) -> Response {
    ws.on_upgrade(|socket| message_handler(socket, user, state))
}

pub fn authenticated_router(state: SharedState) -> Router<SharedState> {
    Router::new()
        .nest("/accounts", accounts::accounts_router())
        .nest("/goals", goals::goals_router())
        .nest("/envelopes", envelopes::envelopes_router())
        .route("/reports", get(dashboard::index))
        .route("/ws", get(websocket_upgrade))
        .route_layer(middleware::from_fn(validate_csrf))
        .route_layer(middleware::from_fn_with_state(state, authenticated))
}
