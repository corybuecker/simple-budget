use crate::{authenticated::UserExtension, SharedState};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    Extension,
};
use bson::serde_helpers::hex_string_as_object_id;
use mongodb::bson::{doc, oid::ObjectId};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug)]
pub struct AccountRecord {
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
    let accounts: mongodb::Collection<AccountRecord> = shared_state
        .mongo
        .database("simple_budget")
        .collection("accounts");

    let filter = doc! {"_id": ObjectId::from_str(&id).unwrap(), "user_id": ObjectId::from_str(&user.id).unwrap()};

    let Ok(account) = accounts.find_one(filter.clone()).await else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    let Some(_) = account else {
        return Err(StatusCode::NOT_FOUND);
    };

    let _ = accounts.delete_one(filter).await;

    Ok(Redirect::to("/accounts").into_response())
}
