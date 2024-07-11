use crate::{authenticated::UserExtension, SharedState};
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    Extension, Form,
};
use bson::oid::ObjectId;
use chrono::{NaiveDateTime, NaiveTime, Utc};
use mongodb::Collection;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tera::Context;
use validator::Validate;

#[derive(Debug, Validate, Deserialize)]
pub struct Goal {
    #[validate(length(min = 5))]
    name: String,
    #[validate(range(min = 0.0))]
    target: f64,
    target_date: chrono::NaiveDate,
    recurrence: String,
}

#[derive(Serialize)]
pub struct GoalRecord {
    name: String,
    target: f64,
    user_id: ObjectId,

    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    target_date: chrono::DateTime<Utc>,

    recurrence: String,
}

#[derive(Debug)]
pub struct Error {
    message: String,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        return (StatusCode::BAD_REQUEST, format!("{:#?}", self)).into_response();
    }
}

impl From<bson::oid::Error> for Error {
    fn from(value: bson::oid::Error) -> Self {
        Error {
            message: value.to_string(),
        }
    }
}

impl From<tera::Error> for Error {
    fn from(value: tera::Error) -> Self {
        Error {
            message: value.to_string(),
        }
    }
}

impl From<mongodb::error::Error> for Error {
    fn from(value: mongodb::error::Error) -> Self {
        Error {
            message: value.to_string(),
        }
    }
}

pub async fn page(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
    headers: HeaderMap,
    form: Form<Goal>,
) -> Result<Response, Error> {
    log::debug!("{:?}", user);
    log::debug!("{:?}", form);

    let mut turbo = false;
    let accept = headers.get("Accept");
    match accept {
        Some(accept) => {
            if accept.to_str().unwrap().contains("turbo") {
                turbo = true;
            }
        }
        _ => {}
    }

    match form.validate() {
        Ok(_) => {}
        Err(validation_errors) => {
            let mut context = Context::new();

            context.insert("errors", &validation_errors.to_string());
            context.insert("name", &form.name);
            context.insert("target", &form.target);
            context.insert("target_date", &form.target_date);
            context.insert("recurrence", &form.recurrence);

            let content = shared_state.tera.render(
                if turbo {
                    "goals/new.turbo.html"
                } else {
                    "goals/new.html"
                },
                &context,
            )?;

            return Ok((StatusCode::BAD_REQUEST, Html::from(content)).into_response());
        }
    }

    let goal_record = GoalRecord {
        name: form.name.to_owned(),
        target: form.target.to_owned(),
        recurrence: form.recurrence.to_owned(),
        target_date: NaiveDateTime::new(form.target_date, NaiveTime::MIN).and_utc(),
        user_id: ObjectId::from_str(&user.id).unwrap(),
    };

    let goals: Collection<GoalRecord> = shared_state
        .mongo
        .database("simple_budget")
        .collection("goals");

    let _ = goals.insert_one(goal_record).await?;

    Ok(Redirect::to("/goals").into_response())
}
