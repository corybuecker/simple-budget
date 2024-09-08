use crate::SharedState;
use axum::{routing::get, Router};
mod create;
mod delete;
mod edit;
mod index;
mod new;
mod update;
use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Validate, Deserialize)]
pub struct GoalForm {
    #[validate(length(min = 5))]
    name: String,
    #[validate(range(min = 0.0))]
    target: f64,
    target_date: chrono::NaiveDate,
    recurrence: String,
}

pub fn goals_router() -> Router<SharedState> {
    Router::new()
        .route("/", get(index::page).post(create::page))
        .route(
            "/:id",
            get(edit::page).put(update::action).delete(delete::action),
        )
        .route("/new", get(new::page))
        .route("/:id/delete", get(delete::modal))
}
