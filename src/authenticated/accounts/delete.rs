use crate::{
    authenticated::{FormError, UserExtension},
    models::account::Account,
    SharedState,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    Extension,
};
use mongodb::bson::{doc, oid::ObjectId};
use std::str::FromStr;
use tracing::info;

pub async fn page(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
    Path(id): Path<String>,
) -> Result<Response, FormError> {
    info!("{:#?}", user);

    let accounts: mongodb::Collection<Account> = shared_state
        .mongo
        .database("simple_budget")
        .collection("accounts");

    let filter = doc! {"_id": ObjectId::from_str(&id).unwrap(), "user_id": ObjectId::from_str(&user.id).unwrap()};
    let account = accounts.find_one(filter.clone()).await?;
    let Some(_) = account else {
        return Ok((StatusCode::NOT_FOUND, Html::from("Not Found")).into_response());
    };
    let _ = accounts.delete_one(filter).await?;

    Ok(Redirect::to("/accounts").into_response())
}
