use crate::{
    authenticated::UserExtension,
    errors::FormError,
    models::{
        goal::Goal,
        user::{GoalHeader, User},
    },
    SharedState,
};
use axum::{
    extract::State,
    response::{Html, IntoResponse, Response},
    Extension,
};
use chrono::Utc;
use std::collections::HashMap;
use tera::Context;

pub async fn page(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
) -> Result<Response, FormError> {
    let mut context = Context::new();
    let mut accumulations: HashMap<String, f64> = HashMap::new();
    let mut days_remainings: HashMap<String, i16> = HashMap::new();

    context.insert("csrf", &user.csrf);

    let user = User::get_by_id(&shared_state.mongo, &user.id)
        .await
        .unwrap();

    let goal_header = &user.preferences.goal_header;

    context.insert(
        "goal_header",
        goal_header.as_ref().unwrap_or(&GoalHeader::Accumulated),
    );

    let goals = Goal::get_by_user_id(&shared_state.mongo, &user._id)
        .await
        .unwrap();

    for goal in &goals {
        accumulations.insert(goal._id.clone(), goal.accumulated());
        days_remainings.insert(
            goal._id.clone(),
            (goal.target_date - Utc::now())
                .num_days()
                .try_into()
                .unwrap(),
        );
    }

    context.insert("goals", &goals);
    context.insert("accumulations", &accumulations);
    context.insert("days_remainings", &days_remainings);

    let content = shared_state.tera.render("goals/index.html", &context)?;

    Ok(Html::from(content).into_response())
}
