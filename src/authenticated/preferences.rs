use crate::{models::user::GoalHeader, SharedState};
use axum::{routing::get, Router};
use serde::Deserialize;
use validator::Validate;
mod index;
mod update;

#[derive(Debug, Validate, Deserialize)]
pub struct PreferencesForm {
    timezone: Option<String>,
    goal_header: Option<GoalHeader>,
    forecast_offset: Option<i64>,
}

pub fn preferences_router() -> Router<SharedState> {
    Router::new().route("/", get(index::action).put(update::action))
}
