use crate::SharedState;
use axum::{
    Router,
    routing::{get, post},
};
mod callback;
mod client;
mod login;
mod token;

pub fn authentication_router() -> Router<SharedState> {
    Router::new()
        .route("/authentication/login", get(login::login))
        .route("/authentication/redirect", get(login::redirect))
        .route("/authentication/token", post(token::token))
        .route("/authentication/callback", get(callback::callback))
}
