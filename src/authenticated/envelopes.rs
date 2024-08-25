use crate::SharedState;
use axum::{routing::get, Router};
use serde::Deserialize;
use validator::Validate;
mod create;
mod delete;
mod edit;
mod index;
mod new;
mod update;

#[derive(Debug, Validate, Deserialize)]
pub struct EnvelopeForm {
    #[validate(length(min = 5))]
    pub name: String,
    #[validate(range(min = 0.0))]
    pub amount: f64,
}

pub fn envelopes_router() -> Router<SharedState> {
    Router::new()
        .route("/", get(index::page).post(create::page))
        .route(
            "/:id",
            get(edit::page).put(update::action).delete(delete::action),
        )
        .route("/new", get(new::page))
}
