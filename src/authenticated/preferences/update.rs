use super::PreferencesForm;
use crate::{
    HandlebarsContext, SharedState,
    authenticated::{UserExtension, dashboard::generate_dashboard_context_for},
    errors::AppResponse,
    models::{
        goal::Goal,
        user::{Preferences, User},
    },
};
use anyhow::anyhow;
use axum::{
    Extension, Form,
    extract::State,
    response::{Html, IntoResponse},
};
use chrono::Utc;
use handlebars::to_json;
use postgres_types::Json;
use rust_decimal::{Decimal, prelude::FromPrimitive};
use tracing::error;

pub async fn action(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
    Extension(mut context): Extension<HandlebarsContext>,
    form: Form<PreferencesForm>,
) -> AppResponse {
    let client = shared_state.pool.get_client().await?;
    let mut user = User::get_by_id(&client, user.id).await?;

    let mut preferences = match user.preferences {
        Some(preferences) => preferences.0.clone(),
        None => Preferences {
            goal_header: None,
            timezone: None,
            forecast_offset: None,
            monthly_income: Some(Decimal::ZERO),
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

    match &form.monthly_income {
        None => {}
        Some(monthly_income) => {
            preferences.monthly_income = Some(
                Decimal::from_f64(*monthly_income)
                    .ok_or_else(|| anyhow!("could not parse decimal"))?,
            )
        }
    };

    user.preferences = Some(Json(preferences.clone()));

    user.update(&client).await?;

    let goal_header = preferences.goal_header.clone();
    let mut accumulations: Vec<Decimal> = Vec::new();
    let mut days_remaining: Vec<i64> = Vec::new();
    let mut per_days: Vec<Decimal> = Vec::new();

    let goals = Goal::get_all(&client, user.id).await?;

    context.insert("goal_header".to_string(), to_json(goal_header));

    for goal in &goals {
        accumulations.push(goal.accumulated_amount);
        per_days.push(goal.accumulated_per_day()?);
        days_remaining.push((goal.target_date - Utc::now()).num_days());
    }

    context.insert("goals".to_string(), to_json(goals));
    context.insert("accumulations".to_string(), to_json(accumulations));
    context.insert("days_remainings".to_string(), to_json(days_remaining));
    context.insert("per_days".to_string(), to_json(per_days));

    let goals_html = shared_state
        .handlebars
        .render("preferences/goals", &context);

    if goals_html.is_err() {
        error!("{:?}", goals_html);
    }

    let goals_html = goals_html?;
    let mut context = HandlebarsContext::new();
    generate_dashboard_context_for(&mut context, &user, &shared_state.pool.get_client().await?)
        .await?;

    let dashboard_content = shared_state.handlebars.render("_dashboard", &context)?;

    context.insert("goals_update".to_string(), to_json(goals_html));
    context.insert("dashboard_update".to_string(), to_json(dashboard_content));
    let html = shared_state
        .handlebars
        .render("preferences/update", &context)?;

    Ok((
        [("content-type", "text/vnd.turbo-stream.html")],
        Html::from(html),
    )
        .into_response())
}
