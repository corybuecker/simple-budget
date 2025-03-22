use super::UserExtension;
use crate::{Section, SharedState};
use axum::{
    Extension, Router,
    extract::Request,
    middleware::{Next, from_fn},
    response::Response,
    routing::get,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tera::Context;

mod create;
mod delete;
mod edit;
mod index;
mod new;
mod update;

fn schema() -> serde_json::Value {
    json!({
        "type": "object",
        "properties": {
            "name": { "type": "string", "minLength": 2 },
            "target": { "type": "number", "minimum": 0 },
            "recurrence": { "enum": ["never", "daily", "weekly", "monthly", "quarterly", "yearly"] },
            "target_date": { "type": "string", "format": "date" }
        },
        "required": [ "name", "target", "recurrence", "target_date" ],
        "additionalProperties": false
    })
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GoalForm {
    name: String,
    target: f64,
    target_date: chrono::NaiveDate,
    recurrence: String,
}

async fn initialize_context(
    Extension(user_extension): Extension<UserExtension>,
    mut request: Request,
    next: Next,
) -> Response {
    let mut context = Context::new();

    context.insert("section", &Section::Goals);
    context.insert("csrf", &user_extension.csrf);
    request.extensions_mut().insert(context);

    next.run(request).await
}

pub fn goals_router() -> Router<SharedState> {
    Router::new()
        .route("/", get(index::page).post(create::page))
        .route(
            "/{id}",
            get(edit::page).put(update::action).delete(delete::action),
        )
        .route("/new", get(new::page))
        .route("/{id}/delete", get(delete::modal))
        .route_layer(from_fn(initialize_context))
}
