use crate::SharedState;
use axum::{routing::get, Router};
mod index;
mod new;

pub fn accounts_router() -> Router<SharedState> {
    Router::new()
        .route("/", get(index::page))
        .route("/new", get(new::page))
}
