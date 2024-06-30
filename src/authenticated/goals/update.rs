use crate::{authenticated::UserExtension, SharedState};
use axum::{
    extract::{Path, State},
    http::StatusCode,
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

pub async fn page(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
    Path(id): Path<String>,
    form: Form<Goal>,
) -> Result<Response, StatusCode> {
    log::debug!("{:?}", user);
    log::debug!("{:?}", form);

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

            let Ok(content) = shared_state.tera.render("goals/edit.html", &context) else {
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            };

            return Ok((StatusCode::BAD_REQUEST, Html::from(content)).into_response());
        }
    }

    let goals: mongodb::Collection<GoalRecord> = shared_state
        .mongo
        .database("simple_budget")
        .collection("goals");

    let filter = doc! {"_id": ObjectId::from_str(&id).unwrap(), "user_id": ObjectId::from_str(&user.id).unwrap()};
    log::debug!("{:?}", filter);

    let Ok(goal) = goals.find_one(filter.clone(), None).await else {
        log::error!("could not find record");
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    let Some(mut goal) = goal else {
        return Err(StatusCode::NOT_FOUND);
    };

    goal.name = form.name.clone();
    goal.target = form.target;
    goal.target_date = NaiveDateTime::new(form.target_date, NaiveTime::MIN).and_utc();
    goal.recurrence = form.recurrence.clone();
    let _ = goals.replace_one(filter, goal, None).await;

    Ok(Redirect::to("/goals").into_response())
}
