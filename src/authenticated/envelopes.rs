use super::UserExtension;
use crate::{Section, SharedState};
use axum::{
    extract::Request,
    middleware::{from_fn, Next},
    response::Response,
    routing::get,
    Extension, Router,
};
use serde::Deserialize;
use tera::Context;
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

async fn initialize_context(
    Extension(user_extension): Extension<UserExtension>,
    mut request: Request,
    next: Next,
) -> Response {
    let mut context = Context::new();

    context.insert("section", &Section::Envelopes);
    context.insert("csrf", &user_extension.csrf);
    request.extensions_mut().insert(context);

    next.run(request).await
}
pub fn envelopes_router() -> Router<SharedState> {
    Router::new()
        .route("/", get(index::page).post(create::page))
        .route(
            "/:id",
            get(edit::page).put(update::action).delete(delete::action),
        )
        .route("/new", get(new::page))
        .route_layer(from_fn(initialize_context))
}
