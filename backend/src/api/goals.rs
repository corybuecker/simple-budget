use crate::SharedState;
use axum::{routing::get, Router};
mod index;

pub fn goals_router() -> Router<SharedState> {
    Router::new().route("/", get(index::index))
}
