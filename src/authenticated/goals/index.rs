use crate::{
    HandlebarsContext, SharedState,
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
use handlebars::to_json;
use rust_decimal::Decimal;
use tokio_postgres::GenericClient;

pub async fn action(
    shared_state: State<SharedState>,
    headers: HeaderMap,
    user: Extension<UserExtension>,
    Extension(context): Extension<HandlebarsContext>,
) -> AppResponse {
    let mut context = context.clone();
    let mut accumulations: Vec<Decimal> = Vec::new();
    let mut days_remaining: Vec<i64> = Vec::new();
    let mut per_days: Vec<Decimal> = Vec::new();

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
        "goal_header".to_string(),
        to_json(goal_header_for_context.or(Some(GoalHeader::Accumulated))),
    );

    let goals = Goal::get_all(shared_state.pool.get().await?.client(), user.id)
        .await
        .unwrap();

    for goal in &goals {
        accumulations.push(goal.accumulated_amount);
        per_days.push(goal.accumulated_per_day()?);
        days_remaining.push((goal.target_date - Utc::now()).num_days());
    }

    context.insert("goals".to_string(), to_json(&goals));
    context.insert("accumulations".to_string(), to_json(&accumulations));
    context.insert("days_remaining".to_string(), to_json(&days_remaining));
    context.insert("per_days".to_string(), to_json(&per_days));

    let response_format = get_response_format(&headers)?;

    match response_format {
        ResponseFormat::Html | ResponseFormat::Turbo => {
            context.insert("partial".to_string(), to_json("goals/index"));

            Ok(generate_response(
                &ResponseFormat::Html,
                shared_state.handlebars.render("layout", &context)?,
                StatusCode::OK,
            ))
        }
        ResponseFormat::Json => {
            // Create a response structure with all the data
            let default_header = Some(GoalHeader::Accumulated);
            let response_data = serde_json::json!({
                "goals": goals,
                "accumulations": accumulations,
                "days_remaining": days_remaining,
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
