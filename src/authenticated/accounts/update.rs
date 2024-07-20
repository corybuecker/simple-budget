use crate::{
    authenticated::{FormError, UserExtension},
    SharedState,
};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    Extension, Form,
};
use bson::serde_helpers::hex_string_as_object_id;
use mongodb::bson::{doc, oid::ObjectId};
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

#[derive(Serialize, Deserialize, Debug)]
pub struct AccountRecord {
    #[serde(with = "hex_string_as_object_id")]
    _id: String,
    name: String,
    amount: f64,
    debt: bool,
    #[serde(with = "hex_string_as_object_id")]
    user_id: String,
}

pub async fn page(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
    Path(id): Path<String>,
    headers: HeaderMap,
    form: Form<Account>,
) -> Result<Response, FormError> {
    log::debug!("{:?}", user);
    log::debug!("{:?}", form);

    let mut turbo = false;
    let accept = headers.get("Accept");
    match accept {
        Some(accept) => {
            if accept.to_str().unwrap().contains("turbo") {
                turbo = true;
            }
        }
        _ => {}
    }
    match form.validate() {
        Ok(_) => {}
        Err(validation_errors) => {
            let mut context = Context::new();

            context.insert("errors", &validation_errors.to_string());
            context.insert("id", &id);
            context.insert("name", &form.name);
            context.insert("amount", &form.amount);
            context.insert("debt", &form.debt);
            let content = shared_state.tera.render(
                if turbo {
                    "accounts/edit.turbo.html"
                } else {
                    "accounts/edit.html"
                },
                &context,
            )?;

            if turbo {
                return Ok((
                    StatusCode::BAD_REQUEST,
                    [("content-type", "text/vnd.turbo-stream.html")],
                    Html::from(content),
                )
                    .into_response());
            } else {
                return Ok((StatusCode::BAD_REQUEST, Html::from(content)).into_response());
            }
        }
    }

    let accounts: mongodb::Collection<AccountRecord> = shared_state
        .mongo
        .database("simple_budget")
        .collection("accounts");

    let filter = doc! {"_id": ObjectId::from_str(&id).unwrap(), "user_id": ObjectId::from_str(&user.id).unwrap()};
    log::debug!("{:?}", filter);

    let account = accounts.find_one(filter.clone()).await?;

    let Some(mut account) = account else {
        return Err(FormError {
            message: "could not find account".to_string(),
        });
    };

    account.name = form.name.clone();
    account.amount = form.amount;
    account.debt = form.debt.or(Some(false)).unwrap();

    let _ = accounts.replace_one(filter, account).await?;

    Ok(Redirect::to("/accounts").into_response())
}
