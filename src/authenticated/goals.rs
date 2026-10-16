mod create;
mod delete;
mod edit;
mod index;
mod new;
mod resets;
mod update;

use super::UserExtension;
use crate::models::goal::Goal;
use crate::models::user::User;
use crate::models::user::preferences::GoalHeader;
use crate::{LayoutCtx, Section, SharedState};
use anyhow::Result;
use askama::Template;
use axum::{
    Extension, Router,
    extract::Request,
    middleware::{Next, from_fn},
    response::Response,
    routing::{get, post},
};
use chrono::Utc;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

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

#[derive(Template)]
#[template(path = "goals/new.html")]
struct NewTemplate {
    layout: LayoutCtx,
    name: String,
    target: String,
    target_date: String,
    recurrence: String,
    errors: String,
}

#[derive(Template)]
#[template(path = "goals/edit.html")]
struct EditTemplate {
    layout: LayoutCtx,
    id: Uuid,
    name: String,
    target: String,
    target_date: String,
    recurrence: String,
    errors: String,
}

/// One row of the goals table, with the values for every possible third column.
pub struct GoalRow {
    pub goal: Goal,
    pub per_day: Decimal,
    pub days_remaining: i64,
}

impl GoalRow {
    pub fn from_goals(goals: Vec<Goal>) -> Result<Vec<GoalRow>> {
        goals
            .into_iter()
            .map(|goal| {
                Ok(GoalRow {
                    per_day: goal.accumulated_per_day()?,
                    days_remaining: (goal.target_date - Utc::now()).num_days(),
                    goal,
                })
            })
            .collect()
    }
}

/// The user's chosen third column for the goals table, defaulting to `Accumulated`.
pub fn goal_header_for(user: &User) -> GoalHeader {
    user.preferences
        .as_ref()
        .and_then(|preferences| preferences.0.goal_header.clone())
        .unwrap_or(GoalHeader::Accumulated)
}

async fn initialize_context(
    Extension(user_extension): Extension<UserExtension>,
    Extension(layout): Extension<LayoutCtx>,
    mut request: Request,
    next: Next,
) -> Response {
    let mut layout = layout.clone();

    layout.section = Section::Goals;
    layout.csrf_token = Some(user_extension.csrf);

    request.extensions_mut().insert(layout);
    next.run(request).await
}

pub fn goals_router() -> Router<SharedState> {
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
        .route("/resets/{recurrence}", post(resets::action))
        .route("/{id}/delete", get(delete::modal).post(delete::action))
        .route_layer(from_fn(initialize_context))
}
