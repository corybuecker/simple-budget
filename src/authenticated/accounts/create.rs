use crate::{authenticated::UserExtension, SharedState};
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    Extension, Form,
};
use mongodb::{
    bson::{doc, oid::ObjectId},
    Collection,
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tera::Context;
use validator::Validate;

#[derive(Debug, Validate, Deserialize)]
pub struct Account {
    #[validate(length(min = 5))]
    name: String,
    #[validate(range(min = 0.0))]
    amount: f64,
    debt: Option<bool>,
}

#[derive(Serialize)]
pub struct AccountRecord {
    name: String,
    amount: f64,
    debt: bool,
    user_id: ObjectId,
}

pub async fn page(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
    Form(form): Form<Account>,
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
            context.insert("debt", &form.debt);

            let Ok(content) = shared_state.tera.render("accounts/new.html", &context) else {
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            };

            return Ok((StatusCode::BAD_REQUEST, Html::from(content)).into_response());
        }
    }

    let account_record = AccountRecord {
        name: form.name.to_owned(),
        amount: form.amount.to_owned(),
        debt: form.debt.or(Some(false)).unwrap(),
        user_id: ObjectId::from_str(&user.id).unwrap(),
    };
    let accounts: Collection<AccountRecord> = shared_state
        .mongo
        .database("simple_budget")
        .collection("accounts");

    let _ = accounts.insert_one(account_record).await;

    Ok(Redirect::to("/accounts").into_response())
}
