use crate::SharedState;
use axum::{
    routing::{get, post},
    Router,
};
mod callback;
mod login;
mod token;

pub fn authentication_router() -> Router<SharedState> {
    Router::new()
        .route("/login", get(login::login))
        .route("/token", post(token::token))
        .route("/redirect", get(login::redirect))
        .route("/callback", get(callback::callback))
}
