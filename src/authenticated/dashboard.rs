use super::UserExtension;
use crate::{errors::FormError, models::user::User, SharedState};
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Extension,
};
use bson::{doc, oid::ObjectId};
use chrono::{DateTime, Datelike, Duration, Local, Months, NaiveTime, TimeDelta, Timelike};
use mongodb::Collection;
use serde::Deserialize;
use std::{ops::Sub, str::FromStr};
use tera::Context;
mod goals;
use chrono_tz::Tz;

pub async fn index(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
) -> Result<Response, FormError> {
    let Ok(user_id) = ObjectId::from_str(&user.id) else {
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

    let preferences = &user.preferences;
    let timezone = preferences.timezone.clone().unwrap_or(String::from("UTC"));
    let timezone: Tz = timezone.parse().unwrap();

    let mut context = Context::new();

    let goals = goals::goals(&shared_state.mongo, &user_id)
        .await
        .unwrap_or(Vec::new());

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

    let envelopes_total = envelopes_total(&shared_state.mongo, &user_id).await;
    let accounts_total = accounts_total(&shared_state.mongo, &user_id).await;

    let remaining_total = accounts_total - envelopes_total - goals_total;

    let now = Local::now().with_timezone(&timezone);
    let tomorrow = (now + Duration::days(1))
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
    context.insert("remaining_days", &remaining_seconds(&timezone).num_days());
    context.insert("remaining_minutes", &duration_until_tomorrow.num_minutes());
    context.insert("remaining_total", &remaining_total);

    let content = shared_state
        .tera
        .render("dashboard.html", &context)
        .unwrap();

    Ok(Html::from(content).into_response())
}

#[derive(Deserialize, Debug)]
struct Envelope {
    amount: f64,
}
#[derive(Deserialize, Debug)]
struct Account {
    amount: f64,
    debt: bool,
}

async fn accounts_total(client: &mongodb::Client, user_id: &ObjectId) -> f64 {
    let collection: Collection<Account> = client.default_database().unwrap().collection("accounts");

    let mut accounts: Vec<Account> = Vec::new();
    match collection.find(doc! {"user_id": user_id}).await {
        Ok(mut cursor) => {
            while cursor.advance().await.unwrap() {
                match cursor.deserialize_current() {
                    Ok(account) => {
                        accounts.push(account);
                    }
                    Err(e) => {
                        log::error!("{}", e);
                    }
                }
            }
        }
        Err(e) => {
            log::error!("{}", e);
        }
    }

    let debt = accounts
        .iter()
        .filter(|a| a.debt)
        .map(|e| e.amount)
        .reduce(|memo, amount| memo + amount)
        .unwrap_or(0.0);
    let non_debt = accounts
        .iter()
        .filter(|a| !a.debt)
        .map(|e| e.amount)
        .reduce(|memo, amount| memo + amount)
        .unwrap_or(0.0);

    non_debt - debt
}
async fn envelopes_total(client: &mongodb::Client, user_id: &ObjectId) -> f64 {
    let collection: Collection<Envelope> =
        client.default_database().unwrap().collection("envelopes");

    let mut envelopes: Vec<Envelope> = Vec::new();
    match collection.find(doc! {"user_id": user_id}).await {
        Ok(mut cursor) => {
            while cursor.advance().await.unwrap() {
                match cursor.deserialize_current() {
                    Ok(envelope) => {
                        envelopes.push(envelope);
                    }
                    Err(e) => {
                        log::error!("{}", e);
                    }
                }
            }
        }
        Err(e) => {
            log::error!("{}", e);
        }
    }

    let total = envelopes
        .iter()
        .map(|e| e.amount)
        .reduce(|memo, amount| memo + amount);

    total.unwrap_or(0.0)
}

fn remaining_seconds(timezone: &Tz) -> TimeDelta {
    let now = Local::now().with_timezone(timezone);
    let end_of_month = end_of_month(timezone).expect("could not determine end of month");
    let end_of_next_month =
        end_of_next_month(timezone).expect("could not determine end of next month");
    let days = end_of_month - now;

    if days.num_days() == 0 {
        end_of_next_month - now
    } else {
        end_of_month - now
    }
}

fn end_of_next_month(timezone: &Tz) -> Result<DateTime<Tz>, String> {
    let now = Local::now()
        .with_timezone(timezone)
        .checked_add_months(Months::new(2))
        .expect("failed to build datetime");
    let now = now.with_hour(0).ok_or("could not set time");
    let now = now?.with_minute(0).ok_or("could not set time");
    let now = now?.with_second(0).ok_or("could not set time");
    let now = now?.with_day0(0).ok_or("could not set day to zero");
    let now = now?.sub(TimeDelta::new(1, 0).unwrap());
    Ok(now)
}
fn end_of_month(timezone: &Tz) -> Result<DateTime<Tz>, String> {
    let now = Local::now()
        .with_timezone(timezone)
        .checked_add_months(Months::new(1))
        .expect("failed to build datetime");
    let now = now.with_hour(0).ok_or("could not set time");
    let now = now?.with_minute(0).ok_or("could not set time");
    let now = now?.with_second(0).ok_or("could not set time");
    let now = now?.with_day0(0).ok_or("could not set day to zero");
    let now = now?.sub(TimeDelta::new(1, 0).unwrap());
    Ok(now)
}
