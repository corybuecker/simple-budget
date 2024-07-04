use crate::{authenticated::UserExtension, SharedState};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    Extension, Form,
};
use bson::{doc, oid::ObjectId};
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

#[derive(Serialize, Deserialize, Debug)]
pub struct EnvelopeRecord {
    name: String,
    amount: f64,
    user_id: ObjectId,
}

pub async fn page(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
    Path(id): Path<String>,
    form: Form<Envelope>,
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
            context.insert("amount", &form.amount);

            let Ok(content) = shared_state.tera.render("envelopes/edit.html", &context) else {
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            };

            return Ok((StatusCode::BAD_REQUEST, Html::from(content)).into_response());
        }
    }

    let envelopes: mongodb::Collection<EnvelopeRecord> = shared_state
        .mongo
        .database("simple_budget")
        .collection("envelopes");

    let filter = doc! {"_id": ObjectId::from_str(&id).unwrap(), "user_id": ObjectId::from_str(&user.id).unwrap()};
    log::debug!("{:?}", filter);

    let Ok(envelope) = envelopes.find_one(filter.clone()).await else {
        log::error!("could not find record");
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    let Some(mut envelope) = envelope else {
        return Err(StatusCode::NOT_FOUND);
    };

    envelope.name = form.name.clone();
    envelope.amount = form.amount;
    let _ = envelopes.replace_one(filter, envelope).await;

    Ok(Redirect::to("/envelopes").into_response())
}
