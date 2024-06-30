use crate::{authenticated::UserExtension, SharedState};
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    Extension, Form,
};
use bson::oid::ObjectId;
use mongodb::Collection;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tera::Context;
use validator::Validate;

#[derive(Debug, Validate, Deserialize)]
pub struct Envelope {
    #[validate(length(min = 5))]
    name: String,
    #[validate(range(min = 0.0))]
    amount: f64,
}

#[derive(Serialize)]
pub struct EnvelopeRecord {
    name: String,
    amount: f64,
    user_id: ObjectId,
}

pub async fn page(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
    form: Form<Envelope>,
) -> Result<Response, StatusCode> {
    log::debug!("{:?}", user);
    log::debug!("{:?}", form);

    match form.validate() {
        Ok(_) => {}
        Err(validation_errors) => {
            let mut context = Context::new();

            context.insert("errors", &validation_errors.to_string());
            context.insert("name", &form.name);
            context.insert("amount", &form.amount);

            let Ok(content) = shared_state.tera.render("envelopes/new.html", &context) else {
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            };

            return Ok((StatusCode::BAD_REQUEST, Html::from(content)).into_response());
        }
    }

    let goal_record = EnvelopeRecord {
        name: form.name.to_owned(),
        amount: form.amount.to_owned(),
        user_id: ObjectId::from_str(&user.id).unwrap(),
    };
    let goals: Collection<EnvelopeRecord> = shared_state
        .mongo
        .database("simple_budget")
        .collection("envelopes");

    let _ = goals.insert_one(goal_record, None).await;

    Ok(Redirect::to("/envelopes").into_response())
}
