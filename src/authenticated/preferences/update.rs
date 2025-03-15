use super::PreferencesForm;
use crate::{
    SharedState,
    authenticated::{UserExtension, dashboard::generate_dashboard_context_for},
    errors::AppResponse,
    models::{
        goal::Goal,
        user::{Preferences, User},
    },
};
use axum::{
    Extension, Form,
    extract::State,
    response::{Html, IntoResponse},
};
use chrono::Utc;
use postgres_types::Json;
use std::collections::HashMap;
use tera::Context;

pub async fn action(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
    form: Form<PreferencesForm>,
) -> AppResponse {
    let mut user = User::get_by_id(&shared_state.client, user.id)
        .await
        .unwrap();

    let mut preferences = match user.preferences {
        Some(preferences) => preferences.0.clone(),
        None => Preferences {
            goal_header: None,
            timezone: None,
            forecast_offset: None,
        },
    };

    if let Some(string) = &form.timezone {
        if string.is_empty() {
            preferences.timezone = None
        } else {
            preferences.timezone = Some(string.clone())
        }
    }

    if let Some(goal_header) = &form.goal_header {
        let goal_header = goal_header.to_owned();
        preferences.goal_header = Some(goal_header);
    }

    match form.forecast_offset {
        None => {}
        Some(forecast_offset) => {
            if forecast_offset + 1 > 3 {
                preferences.forecast_offset = Some(1)
            } else {
                preferences.forecast_offset = Some(forecast_offset + 1)
            }
        }
    };

    user.preferences = Some(Json(preferences.clone()));
    user.update(&shared_state.client).await?;

    let tera = &shared_state.tera;
    let mut goals_context = Context::new();
    let goal_header = preferences.goal_header.clone();
    let mut accumulations: HashMap<i32, f64> = HashMap::new();
    let mut days_remainings: HashMap<i32, i64> = HashMap::new();
    let mut per_days: HashMap<i32, f64> = HashMap::new();
    let goals = Goal::get_all(&shared_state.client, user.id).await.unwrap();

    goals_context.insert("goal_header", &goal_header);

    for goal in &goals {
        accumulations.insert(goal.id.unwrap(), goal.accumulated());
        per_days.insert(goal.id.unwrap(), goal.accumulated_per_day());
        days_remainings.insert(goal.id.unwrap(), (goal.target_date - Utc::now()).num_days());
    }

    goals_context.insert("goals", &goals);
    goals_context.insert("accumulations", &accumulations);
    goals_context.insert("days_remainings", &days_remainings);
    goals_context.insert("per_days", &per_days);
    goals_context.insert("goals", &goals);

    let goals_html = tera.render("goals/_table.html", &goals_context)?;

    let dashboard_context = generate_dashboard_context_for(&user, &shared_state.client).await;

    let dashboard_content = shared_state
        .tera
        .render("_dashboard.html", &dashboard_context)
        .unwrap();

    let mut context = Context::new();
    context.insert("goals_update", &goals_html);
    context.insert("dashboard_update", &dashboard_content);
    let html = tera.render("preferences/update.html", &context)?;

    Ok((
        [("content-type", "text/vnd.turbo-stream.html")],
        Html::from(html),
    )
        .into_response())
}
