mod index;
mod update;

use super::UserExtension;
use crate::{LayoutCtx, Section, SharedState, models::user::preferences::GoalHeader};
use axum::{
    Extension, Router,
    extract::Request,
    middleware::{Next, from_fn},
    response::Response,
    routing::get,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PreferencesForm {
    timezone: Option<String>,
    goal_header: Option<GoalHeader>,
    forecast_offset: Option<i64>,
    monthly_income: Option<f64>,
    accelerate_goals: Option<String>,
    accelerate_non_monthly: Option<String>,
}

async fn initialize_context(
    Extension(user_extension): Extension<UserExtension>,
    Extension(layout): Extension<LayoutCtx>,
    mut request: Request,
    next: Next,
) -> Response {
    let mut layout = layout.clone();

    layout.section = Section::Preferences;
    layout.csrf_token = Some(user_extension.csrf);

    request.extensions_mut().insert(layout);

    next.run(request).await
}
pub fn preferences_router() -> Router<SharedState> {
    Router::new()
        .route(
            "/",
            get(index::action).post(update::action).put(update::action),
        )
        .route_layer(from_fn(initialize_context))
}
