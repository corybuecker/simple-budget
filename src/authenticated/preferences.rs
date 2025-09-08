mod index;
mod update;

use super::UserExtension;
use crate::{HandlebarsContext, Section, SharedState, models::user::GoalHeader};
use axum::{
    Extension, Router,
    extract::Request,
    middleware::{Next, from_fn},
    response::Response,
    routing::get,
};
use handlebars::to_json;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PreferencesForm {
    timezone: Option<String>,
    goal_header: Option<GoalHeader>,
    forecast_offset: Option<i64>,
    monthly_income: Option<f64>,
}

async fn initialize_context(
    Extension(user_extension): Extension<UserExtension>,
    Extension(context): Extension<HandlebarsContext>,
    mut request: Request,
    next: Next,
) -> Response {
    let mut context = context.clone();

    context.insert("section".to_string(), to_json(Section::Preferences));
    context.insert("csrf".to_string(), to_json(user_extension.csrf));

    request.extensions_mut().insert(context);

    next.run(request).await
}
pub fn preferences_router() -> Router<SharedState> {
    Router::new()
        .route("/", get(index::action).put(update::action))
        .route_layer(from_fn(initialize_context))
}
