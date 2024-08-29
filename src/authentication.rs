use crate::SharedState;
use axum::{routing::get, Router};
mod callback;
mod login;

pub fn authentication_router() -> Router<SharedState> {
    Router::new()
        .route("/login", get(login::login))
        .route("/redirect", get(login::redirect))
        .route("/callback", get(callback::callback))
}
