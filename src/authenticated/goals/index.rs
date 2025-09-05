use crate::{
    SharedState,
    authenticated::UserExtension,
    errors::AppResponse,
    models::{
        goal::Goal,
        user::{GoalHeader, User},
    },
    utilities::responses::{ResponseFormat, generate_response, get_response_format},
};
use axum::{
    Extension, Json,
    extract::State,
    http::{HeaderMap, StatusCode},
};
use chrono::Utc;
use rust_decimal::Decimal;
use std::collections::HashMap;
use tera::Context;
use tokio_postgres::GenericClient;
use tracing::debug;

pub async fn action(
    shared_state: State<SharedState>,
    headers: HeaderMap,
    user: Extension<UserExtension>,
    Extension(mut context): Extension<Context>,
) -> AppResponse {
    let mut accumulations: HashMap<i32, Decimal> = HashMap::new();
    let mut days_remainings: HashMap<i32, i16> = HashMap::new();
    let mut per_days: HashMap<i32, Decimal> = HashMap::new();

    let user = User::get_by_id(shared_state.pool.get().await?.client(), user.id)
        .await
        .unwrap();

    let goal_header = match user.preferences {
        Some(preferences) => preferences.0.goal_header,
        None => Some(GoalHeader::Accumulated),
    };

    // Use a cloned value for the context to avoid the move issue
    let goal_header_for_context = goal_header.clone();
    context.insert(
        "goal_header",
        &goal_header_for_context.or(Some(GoalHeader::Accumulated)),
    );

    let goals = Goal::get_all(shared_state.pool.get().await?.client(), user.id)
        .await
        .unwrap();

    for goal in &goals {
        accumulations.insert(goal.id.unwrap(), goal.accumulated_amount);
        per_days.insert(goal.id.unwrap(), goal.accumulated_per_day()?);
        days_remainings.insert(
            goal.id.unwrap(),
            (goal.target_date - Utc::now())
                .num_days()
                .try_into()
                .unwrap(),
        );
    }

    context.insert("goals", &goals);
    context.insert("accumulations", &accumulations);
    context.insert("days_remainings", &days_remainings);
    context.insert("per_days", &per_days);

    let response_format = get_response_format(&headers)?;

    let content = shared_state.tera.render("goals/index.html", &context);
    debug!("{:#?}", content);

    match response_format {
        ResponseFormat::Turbo | ResponseFormat::Html => Ok(generate_response(
            &ResponseFormat::Html,
            shared_state.tera.render("goals/index.html", &context)?,
            StatusCode::OK,
        )),
        ResponseFormat::Json => {
            // Create a response structure with all the data
            let default_header = Some(GoalHeader::Accumulated);
            let response_data = serde_json::json!({
                "goals": goals,
                "accumulations": accumulations,
                "days_remainings": days_remainings,
                "per_days": per_days,
                "goal_header": goal_header.clone().or(default_header),
            });

            Ok(generate_response(
                &response_format,
                Json(response_data),
                StatusCode::OK,
            ))
        }
    }
}
