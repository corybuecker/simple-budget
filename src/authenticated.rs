use crate::{SharedState, models::user::Session};
use axum::{
    Extension, Router,
    extract::{FromRequest, State},
    http::{Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Redirect, Response},
    routing::get,
};
use axum_extra::extract::{
    SignedCookieJar,
    cookie::{Cookie, SameSite},
};
use serde::Deserialize;
use uuid::Uuid;

pub mod accounts;
mod dashboard;
mod envelopes;
mod goals;
mod preferences;

#[derive(Debug, Clone)]
pub struct UserExtension {
    pub id: Uuid,
    pub csrf: String,
}

#[derive(Deserialize, Debug)]
struct CsrfOnlyForm {
    csrf: String,
}

async fn validate_csrf(
    user: Extension<UserExtension>,
    request: axum::extract::Request,
    next: Next,
) -> Result<Response<axum::body::Body>, StatusCode> {
    let method = request.method();

    match method {
        &Method::POST | &Method::PUT | &Method::PATCH | &Method::DELETE => {
            let (parts, body) = request.into_parts();
            let bytes = axum::body::to_bytes(body, 4096)
                .await
                .map_err(|_e| StatusCode::BAD_REQUEST)?;

            let probe = axum::extract::Request::from_parts(
                parts.clone(),
                axum::body::Body::from(bytes.clone()),
            );

            let probe = axum::extract::Form::<CsrfOnlyForm>::from_request(probe, &())
                .await
                .map_err(|e| {
                    tracing::error!("{}", e);
                    StatusCode::BAD_REQUEST
                })?;

            if probe.csrf.eq(&user.csrf) {
                return Ok(next
                    .run(axum::extract::Request::from_parts(
                        parts,
                        axum::body::Body::from(bytes.clone()),
                    ))
                    .await);
            }

            Err(StatusCode::BAD_REQUEST)
        }
        _ => Ok(next.run(request).await),
    }
}

async fn authenticated(
    State(state): State<SharedState>,
    jar: SignedCookieJar,
    mut request: axum::extract::Request,
    next: Next,
) -> Result<(SignedCookieJar, Response), StatusCode> {
    let Some(session_id) = jar.get("session_id") else {
        let redirect_cookie = Cookie::build(("redirect_to", request.uri().path().to_owned()))
            .expires(None)
            .http_only(true)
            .path("/authentication")
            .same_site(SameSite::Lax)
            .secure(true)
            .build();

        return Ok((
            jar.add(redirect_cookie),
            Redirect::temporary("authentication/login").into_response(),
        ));
    };

    let session_id = session_id.value();

    let session = Session::get_by_id(&state.pool.get_client().await.unwrap(), session_id).await;

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
            .secure(true)
            .build();

        Ok((jar.add(cookie), next.run(request).await))
    } else {
        let redirect_cookie = Cookie::build(("redirect_to", request.uri().path().to_owned()))
            .expires(None)
            .http_only(true)
            .path("/authentication")
            .same_site(SameSite::Lax)
            .secure(true)
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
