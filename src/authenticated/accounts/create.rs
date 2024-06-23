use std::str::FromStr;

use crate::{authenticated::UserExtension, SharedState};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    Extension, Form,
};
use mongodb::{
    bson::{doc, oid::ObjectId},
    Collection,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Account {
    name: String,
    amount: f64,
}

#[derive(Serialize)]
pub struct AccountRecord {
    name: String,
    amount: f64,
    user_id: ObjectId,
}

pub async fn page(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
    form: Form<Account>,
) -> Result<Response, StatusCode> {
    log::debug!("{:?}", user);
    log::debug!("{:?}", form);

    let account_record = AccountRecord {
        name: form.name.to_owned(),
        amount: form.amount.to_owned(),
        user_id: ObjectId::from_str(&user.id).unwrap(),
    };
    let accounts: Collection<AccountRecord> = shared_state
        .mongo
        .database("simple_budget")
        .collection("accounts");

    let _ = accounts.insert_one(account_record, None).await;

    Ok(Redirect::to("/accounts").into_response())
}
