mod create;
mod delete;
mod edit;
mod index;
mod new;
mod resets;
mod update;

use super::UserExtension;
use crate::HandlebarsContext;
use crate::models::goal::Goal;
use crate::models::user::User;
use crate::models::user::preferences::GoalHeader;
use crate::{Section, SharedState};
use anyhow::Result;
use axum::{
    Extension, Router,
    extract::Request,
    middleware::{Next, from_fn},
    response::Response,
    routing::{get, post},
};
use chrono::Utc;
use handlebars::to_json;
use rust_database_common::GenericClient;
use rust_decimal::Decimal;
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

pub async fn generate_goal_index_context_for(
    context: &mut HandlebarsContext,
    user: &User,
    client: &impl GenericClient,
) -> Result<()> {
    let mut accumulations: Vec<Decimal> = Vec::new();
    let mut days_remaining: Vec<i64> = Vec::new();
    let mut per_days: Vec<Decimal> = Vec::new();

    let goal_header = match &user.preferences {
        Some(preferences) => &preferences.0.goal_header,
        None => &Some(GoalHeader::Accumulated),
    };

    // Use a cloned value for the context to avoid the move issue
    let goal_header_for_context = goal_header.clone();
    context.insert(
        "goal_header".to_string(),
        to_json(goal_header_for_context.or(Some(GoalHeader::Accumulated))),
    );

    let goals = Goal::get_all(client, user.id).await.unwrap();

    for goal in &goals {
        accumulations.push(goal.accumulated_amount);
        per_days.push(goal.accumulated_per_day()?);
        days_remaining.push((goal.target_date - Utc::now()).num_days());
    }

    context.insert("goals".to_string(), to_json(&goals));
    context.insert("accumulations".to_string(), to_json(&accumulations));
    context.insert("days_remaining".to_string(), to_json(&days_remaining));
    context.insert("per_days".to_string(), to_json(&per_days));

    Ok(())
}
