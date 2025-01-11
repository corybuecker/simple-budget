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
        .route("/authentication/login", get(login::login))
        .route("/authentication/token", post(token::token))
        .route("/authentication/redirect", get(login::redirect))
        .route("/authentication/callback", get(callback::callback))
}
