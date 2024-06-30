use crate::{authenticated::UserExtension, SharedState};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Extension,
};
use bson::{doc, oid::ObjectId, serde_helpers::hex_string_as_object_id};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tera::Context;

#[derive(Serialize, Deserialize, Debug)]
struct Envelope {
    #[serde(with = "hex_string_as_object_id")]
    _id: String,
    name: String,
    amount: f64,
    #[serde(with = "hex_string_as_object_id")]
    user_id: String,
}

pub async fn page(
    shared_state: State<SharedState>,
    Path(id): Path<String>,
    user: Extension<UserExtension>,
) -> Result<Response, StatusCode> {
    log::debug!("{:?}", user);

    let envelopes: mongodb::Collection<Envelope> = shared_state
        .mongo
        .database("simple_budget")
        .collection("envelopes");

    let Ok(envelope) = envelopes
        .find_one(
            doc! {"_id": ObjectId::from_str(&id).unwrap(), "user_id": ObjectId::from_str(&user.id).unwrap()},
            None,
        )
        .await
    else {
        return Err(StatusCode::NOT_FOUND);
    };

    let Some(envelope) = envelope else {
        return Err(StatusCode::NOT_FOUND);
    };

    let mut context = Context::new();

    context.insert("id", &envelope._id);
    context.insert("name", &envelope.name);
    context.insert("amount", &envelope.amount);

    let Ok(content) = shared_state.tera.render("envelopes/edit.html", &context) else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    Ok(Html::from(content).into_response())
}
