use super::PreferencesForm;
use crate::{
    HandlebarsContext, SharedState,
    authenticated::{UserExtension, dashboard::generate_dashboard_context_for},
    errors::AppResponse,
    models::user::{Preferences, User},
};
use anyhow::anyhow;
use axum::{
    Extension, Form,
    extract::State,
    response::{Html, IntoResponse},
};
use postgres_types::Json;
use rust_decimal::{Decimal, prelude::FromPrimitive};

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

    generate_dashboard_context_for(&mut context, &user, &client).await?;

    let html = shared_state
        .handlebars
        .render("preferences/update", &context)?;

    Ok((
        [("content-type", "text/vnd.turbo-stream.html")],
        Html::from(html),
    )
        .into_response())
}
