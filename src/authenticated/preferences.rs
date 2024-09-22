use crate::{models::user::GoalHeader, Section, SharedState};
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

use super::UserExtension;
mod index;
mod update;

#[derive(Debug, Validate, Deserialize)]
pub struct PreferencesForm {
    timezone: Option<String>,
    goal_header: Option<GoalHeader>,
    forecast_offset: Option<i64>,
}

async fn initialize_context(
    Extension(user_extension): Extension<UserExtension>,
    mut request: Request,
    next: Next,
) -> Response {
    let mut context = Context::new();

    context.insert("section", &Section::Preferences);
    context.insert("csrf", &user_extension.csrf);
    request.extensions_mut().insert(context);

    next.run(request).await
}
pub fn preferences_router() -> Router<SharedState> {
    Router::new()
        .route("/", get(index::action).put(update::action))
        .route_layer(from_fn(initialize_context))
}
