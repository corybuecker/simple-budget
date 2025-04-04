use crate::{SharedState, models::user::Session};
use axum::{
    Extension, Router,
    extract::{Request, State, WebSocketUpgrade, ws::WebSocket},
    http::{HeaderMap, Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Redirect, Response},
    routing::get,
};
use axum_extra::extract::{
    SignedCookieJar,
    cookie::{Cookie, SameSite},
};
use futures_util::{SinkExt, stream::StreamExt};
use std::env;
use tokio::{spawn, sync::watch};
use tokio_postgres::GenericClient;
use tracing::debug;

pub mod accounts;
mod dashboard;
mod envelopes;
mod goals;
mod preferences;

#[derive(Debug, Clone)]
pub struct UserExtension {
    pub id: i32,
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
    let secure = env::var("SECURE").unwrap_or("false".to_string()) == "true";

    let Some(session_id) = jar.get("session_id") else {
        let redirect_cookie = Cookie::build(("redirect_to", request.uri().path().to_owned()))
            .expires(None)
            .http_only(true)
            .path("/authentication")
            .same_site(SameSite::Lax)
            .secure(secure)
            .build();

        return Ok((
            jar.add(redirect_cookie),
            Redirect::temporary("authentication/login").into_response(),
        ));
    };

    let session_id = session_id.value();

    let session = Session::get_by_id(state.pool.get().await.unwrap().client(), session_id).await;

    if let Ok(session) = session {
        let (tx, rx) = watch::channel(String::new());
        request.extensions_mut().insert(UserExtension {
            id: session.user_id,
            csrf: session.csrf.clone(),
            channel_sender: tx,
            channel_receiver: rx,
        });

        let cookie = Cookie::build(("session_id", session_id.to_string()))
            .expires(None)
            .http_only(true)
            .path("/")
            .same_site(SameSite::Strict)
            .secure(secure)
            .build();

        Ok((jar.add(cookie), next.run(request).await))
    } else {
        let redirect_cookie = Cookie::build(("redirect_to", request.uri().path().to_owned()))
            .expires(None)
            .http_only(true)
            .path("/authentication")
            .same_site(SameSite::Lax)
            .secure(secure)
            .build();

        Ok((
            jar.add(redirect_cookie),
            Redirect::temporary("authentication/login").into_response(),
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
        .nest("/preferences", preferences::preferences_router())
        .nest("/envelopes", envelopes::envelopes_router())
        .route("/", get(dashboard::index))
        .route("/ws", get(websocket_upgrade))
        .route_layer(middleware::from_fn(validate_csrf))
        .route_layer(middleware::from_fn_with_state(state, authenticated))
}
