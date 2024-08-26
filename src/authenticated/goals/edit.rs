use crate::{authenticated::UserExtension, errors::FormError, models::goal::Goal, SharedState};
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
) -> Result<Response, FormError> {
    let goals: mongodb::Collection<Goal> = shared_state
        .mongo
        .default_database()
        .unwrap()
        .collection("goals");

    let goal = goals
        .find_one(
            doc! {"_id": ObjectId::from_str(&id).unwrap(), "user_id": ObjectId::from_str(&user.id).unwrap()})
        .await?;

    let Some(goal) = goal else {
        return Err(FormError {
            message: "could not find goal".to_string(),
            status_code: Some(StatusCode::NOT_FOUND),
        });
    };

    let mut context = Context::new();

    context.insert("csrf", &user.csrf);
    context.insert("id", &goal._id);
    context.insert("name", &goal.name);
    context.insert("target", &goal.target);
    context.insert("target_date", &goal.target_date.date_naive());
    context.insert("recurrence", &goal.recurrence);

    let content = shared_state.tera.render("goals/edit.html", &context)?;

    Ok(Html::from(content).into_response())
}
