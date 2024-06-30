use super::UserExtension;
use crate::SharedState;
use axum::{
    extract::{FromRef, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Extension,
};
use bson::{doc, oid::ObjectId};
use chrono::{
    DateTime, Datelike, FixedOffset, Local, Months, NaiveDateTime, TimeDelta, TimeZone, Timelike,
    Utc,
};
use mongodb::Collection;
use serde::Deserialize;
use std::{ops::Sub, str::FromStr};
use tera::Context;
mod goals;

pub async fn index(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
) -> Result<Response, StatusCode> {
    let Ok(user_id) = ObjectId::from_str(&user.id) else {
        return Err(StatusCode::FORBIDDEN);
    };

    let mut context = Context::new();

    let Ok(end_of_month) = end_of_month() else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    let goals = goals::goals(&shared_state.mongo, &user_id).await.unwrap();
    let goals_total = goals
        .iter()
        .map(|g| g.accumulated())
        .reduce(|memo, a| memo + a)
        .unwrap();

    context.insert("end_of_month", &end_of_month);
    context.insert("remaining_seconds", &remaining_seconds().num_seconds());

    let envelopes_total = envelopes_total(&shared_state.mongo, &user_id).await;
    context.insert("envelopes_total", &envelopes_total);
    let accounts_total = accounts_total(&shared_state.mongo, &user_id).await;
    context.insert("accounts_total", &accounts_total);
    context.insert("goals_total", &goals_total);

    match shared_state.tera.render("dashboard.html", &context) {
        Ok(content) => Ok(Html::from(content).into_response()),
        Err(e) => {
            log::error!("{}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
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
    let collection: Collection<Account> = client.database("simple_budget").collection("accounts");

    let mut accounts: Vec<Account> = Vec::new();
    match collection.find(doc! {"user_id": user_id}, None).await {
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
        .filter(|a| a.debt == true)
        .map(|e| e.amount)
        .reduce(|memo, amount| memo + amount)
        .or(Some(0.0))
        .unwrap();
    let non_debt = accounts
        .iter()
        .filter(|a| a.debt == false)
        .map(|e| e.amount)
        .reduce(|memo, amount| memo + amount)
        .or(Some(0.0))
        .unwrap();

    non_debt - debt
}
async fn envelopes_total(client: &mongodb::Client, user_id: &ObjectId) -> f64 {
    let collection: Collection<Envelope> = client.database("simple_budget").collection("envelopes");

    let mut envelopes: Vec<Envelope> = Vec::new();
    match collection.find(doc! {"user_id": user_id}, None).await {
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

    total.or(Some(0.0)).unwrap()
}

fn remaining_seconds() -> TimeDelta {
    let now = Local::now();
    let end_of_month = end_of_month().expect("could not determine end of month");
    end_of_month - now
}

fn end_of_month() -> Result<DateTime<Local>, String> {
    let now = Local::now()
        .checked_add_months(Months::new(1))
        .expect("failed to build datetime");
    let now = now.with_hour(0).ok_or("could not set time");
    let now = now?.with_minute(0).ok_or("could not set time");
    let now = now?.with_second(0).ok_or("could not set time");
    let now = now?.with_day0(0).ok_or("could not set day to zero");
    let now = now?.sub(TimeDelta::new(1, 0).unwrap());
    Ok(now)
}
