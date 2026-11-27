mod create;
mod delete;
mod edit;
mod index;
mod new;
mod update;

use super::UserExtension;
use crate::{LayoutCtx, Section, SharedState};
use askama::Template;
use axum::{
    Extension, Router,
    extract::Request,
    middleware::{Next, from_fn},
    response::Response,
    routing::get,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

fn schema() -> serde_json::Value {
    json!({
        "type": "object",
        "properties": {
            "name": { "type": "string", "minLength": 2 },
            "amount": { "type": "number", "minimum": 0 },
        },
        "required": [ "name", "amount" ],
        "additionalProperties": false
    })
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EnvelopeForm {
    pub name: String,
    pub amount: f64,
}

#[derive(Template)]
#[template(path = "envelopes/new.html")]
struct NewTemplate {
    layout: LayoutCtx,
    name: String,
    amount: String,
    errors: String,
}

#[derive(Template)]
#[template(path = "envelopes/edit.html")]
struct EditTemplate {
    layout: LayoutCtx,
    id: Uuid,
    name: String,
    amount: String,
    errors: String,
}

async fn initialize_context(
    Extension(user_extension): Extension<UserExtension>,
    Extension(layout): Extension<LayoutCtx>,
    mut request: Request,
    next: Next,
) -> Response {
    let mut layout = layout.clone();

    layout.section = Section::Envelopes;
    layout.csrf_token = Some(user_extension.csrf);

    request.extensions_mut().insert(layout);
    next.run(request).await
}

pub fn envelopes_router() -> Router<SharedState> {
    Router::new()
        .route("/", get(index::action).post(create::action))
        .route(
            "/{id}",
            get(edit::action)
                .post(update::action)
                .put(update::action)
                .delete(delete::action),
        )
        .route("/new", get(new::action))
        .route("/{id}/delete", get(delete::modal).post(delete::action))
        .route_layer(from_fn(initialize_context))
}
