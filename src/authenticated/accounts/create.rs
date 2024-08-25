use super::AccountForm;
use crate::{
    authenticated::{FormError, UserExtension},
    models::account::Account,
    SharedState,
};
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    Extension, Form,
};
use mongodb::{bson::oid::ObjectId, Collection};
use std::str::FromStr;
use tera::Context;
use validator::Validate;

pub async fn page(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
    headers: HeaderMap,
    Form(form): Form<AccountForm>,
) -> Result<Response, FormError> {
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
                    "accounts/form.turbo.html"
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

    let account_record = Account {
        _id: ObjectId::new().to_string(),
        name: form.name.to_owned(),
        amount: form.amount.to_owned(),
        debt: form.debt.or(Some(false)).unwrap(),
        user_id: ObjectId::from_str(&user.id).unwrap().to_string(),
    };

    let accounts: Collection<Account> = shared_state
        .mongo
        .database("simple_budget")
        .collection("accounts");

    let _ = accounts.insert_one(account_record).await?;

    Ok(Redirect::to("/accounts").into_response())
}
