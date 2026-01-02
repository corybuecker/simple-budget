mod create;
mod delete;
mod edit;
mod index;
mod new;
mod resets;
mod update;

use super::UserExtension;
use crate::HandlebarsContext;
use crate::{Section, SharedState};
use axum::{
    Extension, Router,
    extract::Request,
    middleware::{Next, from_fn},
    response::Response,
    routing::{get, post},
};
use handlebars::to_json;
use serde::{Deserialize, Serialize};
use serde_json::json;

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
    Extension(context): Extension<HandlebarsContext>,
    mut request: Request,
    next: Next,
) -> Response {
    let mut context = context.clone();

    context.insert("section".to_string(), to_json(Section::Goals));
    context.insert("csrf".to_string(), to_json(user_extension.csrf));

    request.extensions_mut().insert(context);

    next.run(request).await
}

pub fn goals_router() -> Router<SharedState> {
    Router::new()
        .route("/", get(index::action).post(create::action))
        .route(
            "/{id}",
            get(edit::action).put(update::action).delete(delete::action),
        )
        .route("/new", get(new::action))
        .route("/resets/{recurrence}", post(resets::action))
        .route("/{id}/delete", get(delete::modal))
        .route_layer(from_fn(initialize_context))
}
