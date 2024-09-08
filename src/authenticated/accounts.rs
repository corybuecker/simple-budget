use crate::SharedState;
mod create;
use axum::{routing::get, Router};
use serde::Deserialize;
use validator::Validate;
mod delete;
mod edit;
mod index;
mod new;
mod update;

#[derive(Debug, Validate, Deserialize)]
pub struct AccountForm {
    #[validate(length(min = 5))]
    name: String,
    #[validate(range(min = 0.0))]
    amount: f64,
    debt: Option<bool>,
}

pub fn accounts_router() -> Router<SharedState> {
    Router::new()
        .route("/", get(index::page).post(create::page))
        .route(
            "/:id",
            get(edit::page).put(update::action).delete(delete::action),
        )
        .route("/new", get(new::page))
        .route("/:id/delete", get(delete::modal))
}
