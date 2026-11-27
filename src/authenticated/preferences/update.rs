use super::PreferencesForm;
use crate::{
    SharedState,
    authenticated::UserExtension,
    errors::AppResponse,
    models::user::{User, preferences::Preferences},
};
use anyhow::anyhow;
use axum::{
    Extension, Form,
    extract::State,
    response::{IntoResponse, Redirect},
};
use postgres_types::Json;
use rust_decimal::{Decimal, prelude::FromPrimitive};

pub async fn action(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
    form: Form<PreferencesForm>,
) -> AppResponse {
    let client = shared_state.pool.get_client().await?;
    let mut user = User::get_by_id(&client, user.id).await?;

    let mut preferences = match user.preferences {
        Some(preferences) => preferences.0.clone(),
        None => Preferences::default(),
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

    preferences.accelerate_goals = None;
    preferences.accelerate_non_monthly = None;

    if form.accelerate_goals.is_some() {
        preferences.accelerate_goals = Some(true);
    }

    if form.accelerate_non_monthly.is_some() {
        preferences.accelerate_non_monthly = Some(true);
    }

    user.preferences = Some(Json(preferences.clone()));
    user.update(&client).await?;

    // The goals table header only submits `goal_header`; send that back to the goals page.
    if form.goal_header.is_some() {
        Ok(Redirect::to("/goals").into_response())
    } else {
        Ok(Redirect::to("/preferences").into_response())
    }
}
