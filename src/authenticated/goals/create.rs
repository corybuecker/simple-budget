use crate::{authenticated::UserExtension, SharedState};
use axum::{
    extract::State,
    http::StatusCode,
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

pub async fn page(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
    form: Form<Goal>,
) -> Result<Response, StatusCode> {
    log::debug!("{:?}", user);
    log::debug!("{:?}", form);

    match form.validate() {
        Ok(_) => {}
        Err(validation_errors) => {
            let mut context = Context::new();

            context.insert("errors", &validation_errors.to_string());
            context.insert("name", &form.name);
            context.insert("target", &form.target);
            context.insert("target_date", &form.target_date);
            context.insert("recurrence", &form.recurrence);

            let Ok(content) = shared_state.tera.render("goals/new.html", &context) else {
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            };

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

    let _ = goals.insert_one(goal_record).await;

    Ok(Redirect::to("/goals").into_response())
}
