use super::UserExtension;
use crate::errors::AppResponse;
use crate::models::goal::Goal;
use crate::models::user::preferences::Preferences;
use crate::utilities::dates::{TimeProvider, TimeUtilities};
use crate::utilities::responses::{ResponseFormat, generate_response, get_response_format};
use crate::{HandlebarsContext, LayoutCtx};
use crate::{Section, SharedState, models::user::User};
use anyhow::{Result, anyhow};
use askama::Template;
use axum::{
    Extension, Json,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use chrono::{Duration, Local, NaiveTime};
use chrono_tz::Tz;
use rust_database_common::GenericClient;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;

#[derive(Template)]
#[template(path = "dashboard.html")]
struct Dashboard {
    pub layout: LayoutCtx,
    pub tomorrow_remaining_total: String,
    pub goals_accumulated_per_day: String,
    pub remaining_days: String,
    pub remaining_minutes: String,
    pub remaining_total: String,
    pub forecast_offset: String,
    pub per_diem: String,
    pub per_diem_diff_monthly: String,
}

pub async fn index(
    shared_state: State<SharedState>,
    headers: HeaderMap,
    user: Extension<UserExtension>,
) -> AppResponse {
    let client = shared_state.pool.get_client().await?;
    let csrf = user.csrf.clone();
    let user = User::get_by_id(&client, user.id).await?;
    let mut dashboard = Dashboard {
        layout: LayoutCtx {
            section: Section::Reports,
            csrf_token: Some(csrf.clone()),
            nonce: "test".to_string(),
        },

        tomorrow_remaining_total: String::new(),
        goals_accumulated_per_day: String::new(),
        remaining_days: String::new(),
        remaining_minutes: String::new(),
        remaining_total: String::new(),
        forecast_offset: String::new(),
        per_diem: String::new(),
        per_diem_diff_monthly: String::new(),
    };
    generate_dashboard_context_for(&mut dashboard, &user, &client).await?;

    let response_format = get_response_format(&headers)?;

    match response_format {
        ResponseFormat::Html => Ok(generate_response(
            &response_format,
            dashboard.to_string(),
            StatusCode::OK,
        )),
        _ => Ok(StatusCode::NOT_ACCEPTABLE.into_response()),
    }
}

async fn generate_dashboard_context_for(
    context: &mut Dashboard,
    user: &User,
    client: &impl GenericClient,
) -> Result<()> {
    let preferences = match &user.preferences {
        Some(preferences) => &preferences.0,
        None => &Preferences {
            goal_header: None,
            timezone: Some(String::from("UTC")),
            forecast_offset: None,
            monthly_income: Some(Decimal::ZERO),
            accelerate_goals: Some(false),
            accelerate_non_monthly: None,
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
        .map(|g| g.accumulated_per_day())
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

    context.tomorrow_remaining_total = tomorrow_remaining_total.to_string();
    context.goals_accumulated_per_day = goals_accumulated.to_string();
    context.remaining_days = remaining_days.to_string();
    context.remaining_minutes = remaining_total.to_string();
    context.forecast_offset = forecast_offset.to_string();
    context.per_diem = per_diem.to_string();
    context.per_diem_diff_monthly = per_diem_diff_monthly.to_string();
    context.tomorrow_remaining_total = tomorrow_remaining_total.to_string();
    context.tomorrow_remaining_total = tomorrow_remaining_total.to_string();

    Ok(())
}
