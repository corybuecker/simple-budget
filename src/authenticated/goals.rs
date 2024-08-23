use crate::SharedState;
use axum::{routing::get, Router};
mod create;
mod delete;
mod edit;
mod index;
mod new;
mod update;

pub fn goals_router() -> Router<SharedState> {
    Router::new()
        .route("/", get(index::page).post(create::page))
        .route(
            "/:id",
            get(edit::page).put(update::action).delete(delete::action),
        )
        .route("/new", get(new::page))
}
