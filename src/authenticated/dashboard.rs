use super::UserExtension;
use crate::errors::AppResponse;
use crate::models::goal::Goal;
use crate::models::user::Preferences;
use crate::utilities::dates::{TimeProvider, TimeUtilities};
use crate::utilities::responses::{ResponseFormat, generate_response, get_response_format};
use crate::{Section, SharedState, models::user::User};
use anyhow::{Result, anyhow};
use axum::{
    Extension, Json,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use chrono::{Duration, Local, NaiveTime};
use chrono_tz::Tz;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use tera::Context;
use tokio_postgres::{Client, GenericClient};

pub async fn index(
    shared_state: State<SharedState>,
    headers: HeaderMap,
    user: Extension<UserExtension>,
) -> AppResponse {
    let csrf = user.csrf.clone();
    let user = User::get_by_id(shared_state.pool.get().await?.client(), user.id).await?;
    let mut context =
        generate_dashboard_context_for(&user, shared_state.pool.get().await?.client()).await?;
    context.insert("csrf", &csrf);
    context.insert("section", &Section::Reports);

    let response_format = get_response_format(&headers)?;

    match response_format {
        ResponseFormat::Html => Ok(generate_response(
            &response_format,
            shared_state.tera.render("dashboard.html", &context)?,
            StatusCode::OK,
        )),
        ResponseFormat::Turbo => Ok(StatusCode::NOT_ACCEPTABLE.into_response()),
        ResponseFormat::Json => {
            // Extract the relevant data from context for JSON response
            let dashboard_data = serde_json::json!({
                "remaining_total": context.get("remaining_total"),
                "tomorrow_remaining_total": context.get("tomorrow_remaining_total"),
                "goals_accumulated_per_day": context.get("goals_accumulated_per_day"),
                "remaining_days": context.get("remaining_days"),
                "remaining_minutes": context.get("remaining_minutes"),
                "forecast_offset": context.get("forecast_offset"),
                "per_diem": context.get("per_diem"),
            });

            Ok(generate_response(
                &response_format,
                Json(dashboard_data),
                StatusCode::OK,
            ))
        }
    }
}

pub async fn generate_dashboard_context_for(user: &User, client: &Client) -> Result<Context> {
    let mut context = Context::new();
    let preferences = match &user.preferences {
        Some(preferences) => &preferences.0,
        None => &Preferences {
            goal_header: None,
            timezone: Some(String::from("UTC")),
            forecast_offset: None,
            monthly_income: Some(Decimal::ZERO),
        },
    };

    let timezone = preferences.timezone.clone().unwrap_or(String::from("UTC"));
    let timezone: Tz = timezone.parse()?;
    let time_provider = TimeProvider {};
    let time_utilities = &TimeUtilities { timezone };

    let monthly_income = preferences.monthly_income.unwrap_or(Decimal::ZERO);
    let length_of_month = time_utilities
        .length_of_month(&time_provider)?
        .num_seconds();
    let monthly_income_per_day = monthly_income
        / Decimal::from_i64(length_of_month).ok_or(anyhow!("could not convert time to decimal"))?
        * Decimal::new(86400, 0);

    let goals = Goal::get_all(client, user.id).await.unwrap_or(vec![]);
    let goals_accumulated = goals
        .iter()
        .map(|g| g.accumulated_per_day(&time_provider))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .reduce(|memo, d| memo + d)
        .unwrap_or(Decimal::ZERO);
    let remaining_total = user.total_balance(client).await?;
    let forecast_offset = preferences.forecast_offset.unwrap_or(1);
    let now = Local::now().with_timezone(&timezone);
    let tomorrow = (now + Duration::days(forecast_offset))
        .with_time(
            NaiveTime::from_hms_opt(0, 0, 0)
                .ok_or_else(|| anyhow!("could not construct datetime"))?,
        )
        .single()
        .ok_or_else(|| anyhow!("more than one possible time"))?;
    let duration_until_tomorrow = tomorrow - now;
    let seconds_until_tomorrow = duration_until_tomorrow.num_seconds() as f64;
    let seconds_until_tomorrow = Decimal::from_f64(seconds_until_tomorrow / 86400.0)
        .ok_or(anyhow!("could not parse decimal"))?;
    let tomorrow_remaining_total = remaining_total - goals_accumulated * seconds_until_tomorrow;
    let remaining_days_in_seconds = time_utilities
        .remaining_length_of_month(&time_provider)?
        .num_seconds();
    let remaining_days_in_seconds =
        Decimal::from_i64(remaining_days_in_seconds).ok_or(anyhow!("could not parse decimal"))?;
    let per_diem = remaining_total / remaining_days_in_seconds * Decimal::new(86400, 0);
    let forecast_offset = Decimal::from_i64(preferences.forecast_offset.unwrap_or(1))
        .ok_or(anyhow!("could not parse decimal"))?;

    let remaining_days = remaining_days_in_seconds / Decimal::new(86400, 0);
    let remaining_days = remaining_days.round_dp(1).to_string();

    let per_diem_diff_monthly = per_diem - monthly_income_per_day;

    context.insert("tomorrow_remaining_total", &tomorrow_remaining_total);
    context.insert("goals_accumulated_per_day", &goals_accumulated);
    context.insert("remaining_days", &remaining_days);
    context.insert("remaining_minutes", &duration_until_tomorrow.num_minutes());
    context.insert("remaining_total", &remaining_total);
    context.insert("forecast_offset", &forecast_offset);
    context.insert("per_diem", &per_diem);
    context.insert("per_diem_diff_monthly", &per_diem_diff_monthly);

    Ok(context)
}
