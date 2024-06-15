use crate::SharedState;
use axum::{routing::get, Router};
mod index;

pub fn accounts_router() -> Router<SharedState> {
    Router::new().route("/", get(index::index))
}
