use crate::errors::FormError;
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
use mongodb::Collection;
use std::str::FromStr;
use tera::Context;

pub async fn page(
    shared_state: State<SharedState>,
    Path(id): Path<String>,
    user: Extension<UserExtension>,
) -> Result<Response, FormError> {
    let accounts_colllection: Collection<Account> = shared_state
        .mongo
        .default_database()
        .unwrap()
        .collection("accounts");

    let account = accounts_colllection
        .find_one(
            doc! {"_id": ObjectId::from_str(&id).unwrap(), "user_id": ObjectId::from_str(&user.id).unwrap()}
        )
        .await?;

    let Some(account) = account else {
        return Err(FormError {
            message: "could not find account".to_owned(),
            status_code: Some(StatusCode::NOT_FOUND),
        });
    };

    let mut context = Context::new();
    context.insert("csrf", &user.csrf);
    context.insert("id", &account._id);
    context.insert("name", &account.name);
    context.insert("amount", &account.amount);
    context.insert("debt", &account.debt);

    let content = shared_state.tera.render("accounts/edit.html", &context)?;

    Ok(Html::from(content).into_response())
}
