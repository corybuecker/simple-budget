use crate::{
    SharedState,
    authenticated::UserExtension,
    errors::AppResponse,
    models::{
        goal::Goal,
        user::{GoalHeader, User},
    },
    utilities::dates::TimeProvider,
};
use axum::{
    Extension,
    extract::State,
    response::{Html, IntoResponse},
};
use chrono::Utc;
use rust_decimal::Decimal;
use std::collections::HashMap;
use tera::Context;
use tokio_postgres::GenericClient;

pub async fn page(
    shared_state: State<SharedState>,
    mut context: Extension<Context>,
    user: Extension<UserExtension>,
) -> AppResponse {
    let time_provider = TimeProvider {};
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

    context.insert(
        "goal_header",
        &goal_header.or(Some(GoalHeader::Accumulated)),
    );

    let goals = Goal::get_all(shared_state.pool.get().await?.client(), user.id)
        .await
        .unwrap();

    for goal in &goals {
        accumulations.insert(goal.id.unwrap(), goal.accumulated_amount);
        per_days.insert(goal.id.unwrap(), goal.accumulated_per_day(&time_provider)?);
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

    let content = shared_state.tera.render("goals/index.html", &context)?;

    Ok(Html::from(content).into_response())
}
