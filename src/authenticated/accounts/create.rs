use crate::{authenticated::UserExtension, SharedState};
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
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

#[derive(Debug)]
pub struct Error {
    message: String,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        return (StatusCode::BAD_REQUEST, format!("{:#?}", self)).into_response();
    }
}

impl From<bson::oid::Error> for Error {
    fn from(value: bson::oid::Error) -> Self {
        Error {
            message: value.to_string(),
        }
    }
}

impl From<tera::Error> for Error {
    fn from(value: tera::Error) -> Self {
        Error {
            message: value.to_string(),
        }
    }
}
impl From<mongodb::error::Error> for Error {
    fn from(value: mongodb::error::Error) -> Self {
        Error {
            message: value.to_string(),
        }
    }
}
pub async fn page(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
    headers: HeaderMap,
    Form(form): Form<Account>,
) -> Result<Response, Error> {
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
            context.insert("name", &form.name);
            context.insert("amount", &form.amount);
            context.insert("debt", &form.debt);

            let content = shared_state.tera.render(
                if turbo {
                    "accounts/new.turbo.html"
                } else {
                    "accounts/new.html"
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

    let _ = accounts.insert_one(account_record).await?;

    Ok(Redirect::to("/accounts").into_response())
}
