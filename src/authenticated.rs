use crate::{
    SharedState,
    models::user::{Session, User},
};
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
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::env;
use tokio::{spawn, sync::watch};
use tracing::debug;

//pub mod accounts;
//mod dashboard;
mod envelopes;
//mod goals;
//mod preferences;

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

    let session_id = session_id.value().parse::<i32>().unwrap();

    let session = Session::get_by_id(&state.client, session_id).await;

    if let Ok(session) = session {
        let (tx, rx) = watch::channel(String::new());
        request.extensions_mut().insert(UserExtension {
            id: session.user_id.to_string(),
            csrf: session.csrf.clone(),
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

pub fn authenticated_router(state: SharedState) -> Router<SharedState> {
    Router::new()
        // .nest("/accounts", accounts::accounts_router())
        // .nest("/goals", goals::goals_router())
        // .nest("/preferences", preferences::preferences_router())
        .nest("/envelopes", envelopes::envelopes_router())
        // .route("/", get(dashboard::index))
        .route_layer(middleware::from_fn(validate_csrf))
        .route_layer(middleware::from_fn_with_state(state, authenticated))
}
