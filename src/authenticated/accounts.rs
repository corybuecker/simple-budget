mod create;
mod delete;
mod edit;
mod index;
mod new;
mod update;

use super::UserExtension;
use crate::{HandlebarsContext, Section, SharedState};
use axum::{
    Extension, Router,
    extract::Request,
    middleware::{Next, from_fn},
    response::Response,
    routing::get,
};
use handlebars::to_json;
use serde::{Deserialize, Serialize};
use serde_json::json;

fn schema() -> serde_json::Value {
    json!({
        "type": "object",
        "properties": {
            "name": { "type": "string", "minLength": 2 },
            "amount": { "type": "number", "minimum": 0 },
            "debt": { "anyOf": [{ "enum": [true] }, { "type": "null" }] }
        },
        "required": [ "name", "amount" ],
        "additionalProperties": false
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountForm {
    pub name: String,
    pub amount: f64,
    pub debt: Option<bool>,
}

async fn initialize_context(
    Extension(user_extension): Extension<UserExtension>,
    Extension(context): Extension<HandlebarsContext>,
    mut request: Request,
    next: Next,
) -> Response {
    let mut context = context.clone();

    context.insert("section".to_string(), to_json(Section::Accounts));
    context.insert("csrf".to_string(), to_json(user_extension.csrf));

    request.extensions_mut().insert(context);
    next.run(request).await
}

pub fn accounts_router() -> Router<SharedState> {
    Router::new()
        .route("/", get(index::action).post(create::action))
        .route(
            "/{id}",
            get(edit::action).put(update::action).delete(delete::action),
        )
        .route("/new", get(new::action))
        .route("/{id}/delete", get(delete::modal))
        .route_layer(from_fn(initialize_context))
}
