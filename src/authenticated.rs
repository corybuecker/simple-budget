use crate::{SharedState, models::user::Session};
use axum::{
    Extension, Router,
    extract::{Request, State},
    http::{HeaderMap, Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Redirect, Response},
    routing::get,
};
use axum_extra::extract::{
    SignedCookieJar,
    cookie::{Cookie, SameSite},
};
use std::env;
use tokio_postgres::GenericClient;

pub mod accounts;
mod dashboard;
mod envelopes;
mod goals;
mod preferences;

#[derive(Debug, Clone)]
pub struct UserExtension {
    pub id: i32,
    pub csrf: String,
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
        request.extensions_mut().insert(UserExtension {
            id: session.user_id,
            csrf: session.csrf.clone(),
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

pub fn authenticated_router(state: SharedState) -> Router<SharedState> {
    Router::new()
        .nest("/accounts", accounts::accounts_router())
        .nest("/goals", goals::goals_router())
        .nest("/preferences", preferences::preferences_router())
        .nest("/envelopes", envelopes::envelopes_router())
        .route("/", get(dashboard::index))
        .route_layer(middleware::from_fn(validate_csrf))
        .route_layer(middleware::from_fn_with_state(state, authenticated))
}
