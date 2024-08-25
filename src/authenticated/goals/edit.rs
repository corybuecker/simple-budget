use crate::{authenticated::UserExtension, models::goal::Goal, SharedState};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Extension,
};
use bson::{doc, oid::ObjectId};
use std::str::FromStr;
use tera::Context;

pub async fn page(
    shared_state: State<SharedState>,
    Path(id): Path<String>,
    user: Extension<UserExtension>,
) -> Result<Response, StatusCode> {
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
