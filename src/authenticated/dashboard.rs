use super::UserExtension;
use crate::models::account::accounts_total_for;
use crate::models::envelope::envelopes_total_for;
use crate::utilities::dates::{TimeProvider, TimeUtilities};
use crate::{errors::FormError, models::user::User, Section, SharedState};
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Extension,
};
use bson::{doc, oid::ObjectId};
use chrono::{Duration, Local, NaiveTime};
use mongodb::Client;
use std::str::FromStr;
use tera::Context;
mod goals;
use chrono_tz::Tz;

pub async fn index(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
) -> Result<Response, FormError> {
    let csrf = user.csrf.clone();
    let Ok(_user_id) = ObjectId::from_str(&user.id) else {
        return Err(FormError {
            message: "forbidden".to_owned(),
            status_code: Some(StatusCode::FORBIDDEN),
        });
    };

    let user = &shared_state
        .mongo
        .default_database()
        .unwrap()
        .collection::<User>("users")
        .find_one(doc! {"_id": ObjectId::from_str(&user.id).unwrap()})
        .await?;

    let Some(user) = user else {
        return Err(FormError {
            message: "forbidden".to_owned(),
            status_code: Some(StatusCode::FORBIDDEN),
        });
    };

    let mut context = generate_dashboard_context_for(user, &shared_state.mongo).await;
    context.insert("csrf", &csrf);
    context.insert("section", &Section::Reports);
    let content = shared_state
        .tera
        .render("dashboard.html", &context)
        .unwrap();

    Ok(Html::from(content).into_response())
}

pub async fn generate_dashboard_context_for(user: &User, client: &Client) -> Context {
    let mut context = Context::new();
    let user_id = ObjectId::from_str(&user._id).unwrap();
    let preferences = &user.preferences;
    let timezone = preferences.timezone.clone().unwrap_or(String::from("UTC"));
    let timezone: Tz = timezone.parse().unwrap();
    let time_provider = TimeProvider {};
    let time_utilities = &TimeUtilities { timezone };

    let goals = goals::goals(client, &user_id).await.unwrap_or(Vec::new());

    let goals_accumulated = goals
        .iter()
        .map(|g| g.accumulated_per_day())
        .reduce(|memo, a| memo + a)
        .unwrap_or(0.0);
    let goals_total = goals
        .iter()
        .map(|g| g.accumulated())
        .reduce(|memo, a| memo + a)
        .unwrap_or(0.0);

    let envelopes_total = envelopes_total_for(&user_id, client).await;
    let accounts_total = accounts_total_for(&user_id, client).await;

    let remaining_total = accounts_total - envelopes_total - goals_total;

    let forecast_offset = user.preferences.forecast_offset.unwrap_or(1);

    let now = Local::now().with_timezone(&timezone);
    let tomorrow = (now + Duration::days(forecast_offset))
        .with_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
        .unwrap();

    let duration_until_tomorrow = tomorrow - now;
    let seconds_until_tomorrow = duration_until_tomorrow.num_seconds() as f64;

    let tomorrow_remaining_total =
        remaining_total - goals_accumulated * (seconds_until_tomorrow / 86400.0);

    context.insert("tomorrow_remaining_total", &tomorrow_remaining_total);
    context.insert("accounts_total", &accounts_total);
    context.insert("envelopes_total", &envelopes_total);
    context.insert("goals_accumulated_per_day", &goals_accumulated);
    context.insert("goals_total", &goals_total);
    context.insert(
        "remaining_days",
        &time_utilities.remaining_seconds(&time_provider).num_days(),
    );
    context.insert("remaining_minutes", &duration_until_tomorrow.num_minutes());
    context.insert("remaining_total", &remaining_total);
    context.insert("forecast_offset", &forecast_offset);

    context
}
