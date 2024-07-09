use crate::{authenticated::UserExtension, SharedState};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Extension,
};
use bson::{doc, oid::ObjectId, serde_helpers::hex_string_as_object_id};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tera::Context;

#[derive(Serialize, Deserialize, Debug)]
struct Goal {
    #[serde(with = "hex_string_as_object_id")]
    _id: String,
    name: String,
    target: f64,
    #[serde(with = "hex_string_as_object_id")]
    user_id: String,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    target_date: chrono::DateTime<Utc>,
    recurrence: String,
}

pub async fn page(
    shared_state: State<SharedState>,
    Path(id): Path<String>,
    user: Extension<UserExtension>,
) -> Result<Response, StatusCode> {
    log::debug!("{:?}", user);

    let goals: mongodb::Collection<Goal> = shared_state
        .mongo
        .database("simple_budget")
        .collection("goals");

    let Ok(goal) = goals
        .find_one(
            doc! {"_id": ObjectId::from_str(&id).unwrap(), "user_id": ObjectId::from_str(&user.id).unwrap()})
        .await
    else {
        return Err(StatusCode::NOT_FOUND);
    };

    let Some(goal) = goal else {
        return Err(StatusCode::NOT_FOUND);
    };

    let mut context = Context::new();

    context.insert("csrf", &user.csrf);
    context.insert("id", &goal._id);
    context.insert("name", &goal.name);
    context.insert("target", &goal.target);
    context.insert("target_date", &goal.target_date.date_naive());
    context.insert("recurrence", &goal.recurrence);

    let Ok(content) = shared_state.tera.render("goals/edit.html", &context) else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    Ok(Html::from(content).into_response())
}
