use crate::models::account::Account;
use crate::{authenticated::UserExtension, SharedState};
use axum::extract::Path;
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Extension,
};
use mongodb::bson::doc;
use mongodb::bson::oid::ObjectId;
use std::str::FromStr;
use tera::Context;

pub async fn page(
    shared_state: State<SharedState>,
    Path(id): Path<String>,
    user: Extension<UserExtension>,
) -> Result<Response, StatusCode> {
    let accounts: mongodb::Collection<Account> = shared_state
        .mongo
        .default_database()
        .unwrap()
        .collection("accounts");

    let Ok(account) = accounts
        .find_one(
            doc! {"_id": ObjectId::from_str(&id).unwrap(), "user_id": ObjectId::from_str(&user.id).unwrap()}
        )
        .await
    else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    let Some(account) = account else {
        return Err(StatusCode::NOT_FOUND);
    };

    let mut context = Context::new();
    context.insert("csrf", &user.csrf);
    context.insert("id", &account._id);
    context.insert("name", &account.name);
    context.insert("amount", &account.amount);
    context.insert("debt", &account.debt);

    let Ok(content) = shared_state.tera.render("accounts/edit.html", &context) else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    Ok(Html::from(content).into_response())
}
