use crate::{authenticated::UserExtension, SharedState};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    Extension,
};
use bson::doc;
use bson::oid::ObjectId;
use bson::serde_helpers::hex_string_as_object_id;
use core::str::FromStr;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct EnvelopeRecord {
    name: String,
    amount: f64,
    #[serde(with = "hex_string_as_object_id")]
    user_id: String,
}

pub async fn page(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
    Path(id): Path<String>,
) -> Result<Response, StatusCode> {
    log::debug!("{:?}", user);
    let envelopes: mongodb::Collection<EnvelopeRecord> = shared_state
        .mongo
        .database("simple_budget")
        .collection("envelopes");

    let filter = doc! {"_id": ObjectId::from_str(&id).unwrap(), "user_id": ObjectId::from_str(&user.id).unwrap()};

    let Ok(envelope) = envelopes.find_one(filter.clone()).await else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    let Some(_) = envelope else {
        return Err(StatusCode::NOT_FOUND);
    };

    let _ = envelopes.delete_one(filter).await;

    Ok(Redirect::to("/envelopes").into_response())
}
