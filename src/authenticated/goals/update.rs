use crate::{authenticated::UserExtension, SharedState};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    Extension, Form,
};
use bson::{doc, oid::ObjectId};
use chrono::{NaiveDateTime, NaiveTime, Utc};
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

#[derive(Serialize, Deserialize, Debug)]
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
    Path(id): Path<String>,
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
            context.insert("id", &id);
            context.insert("name", &form.name);
            context.insert("target", &form.target);
            context.insert("target_date", &form.target_date);
            context.insert("recurrence", &form.recurrence);

            let content = shared_state.tera.render("goals/edit.html", &context)?;

            return Ok((StatusCode::BAD_REQUEST, Html::from(content)).into_response());
        }
    }

    let goals: mongodb::Collection<GoalRecord> = shared_state
        .mongo
        .database("simple_budget")
        .collection("goals");

    let filter = doc! {"_id": ObjectId::from_str(&id).unwrap(), "user_id": ObjectId::from_str(&user.id).unwrap()};
    log::debug!("{:?}", filter);

    let goal = goals.find_one(filter.clone()).await?;

    let Some(mut goal) = goal else {
        return Err(Error {
            message: "could not find goal".to_string(),
        });
    };

    goal.name = form.name.clone();
    goal.target = form.target;
    goal.target_date = NaiveDateTime::new(form.target_date, NaiveTime::MIN).and_utc();
    goal.recurrence = form.recurrence.clone();
    let _ = goals.replace_one(filter, goal).await?;

    Ok(Redirect::to("/goals").into_response())
}
