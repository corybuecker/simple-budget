use crate::{
    authenticated::{FormError, UserExtension},
    models::account::Account,
    SharedState,
};
use axum::{
    extract::State,
    response::{Html, IntoResponse, Response},
    Extension,
};
use log::debug;
use log::error;
use mongodb::bson::{doc, oid::ObjectId};
use std::str::FromStr;
use tera::Context;

pub async fn page(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
) -> Result<Response, FormError> {
    debug!("{:#?}", user);

    let user_id = ObjectId::from_str(&user.id)?;

    let collection = shared_state
        .mongo
        .database("simple_budget")
        .collection::<Account>("accounts");

    let mut context = Context::new();
    context.insert("csrf", &user.csrf);

    let mut accounts: Vec<Account> = Vec::new();

    match collection.find(doc! {"user_id": &user_id}).await {
        Ok(mut cursor) => {
            while cursor.advance().await.unwrap() {
                match cursor.deserialize_current() {
                    Ok(account) => {
                        accounts.push(account);
                    }
                    Err(e) => {
                        log::error!("{}", e);
                    }
                }
            }
            context.insert("accounts", &accounts);
        }
        Err(e) => {
            error!("{}", e);
            context.insert("accounts", &accounts);
        }
    }

    let content = shared_state.tera.render("accounts/index.html", &context)?;

    Ok(Html::from(content).into_response())
}
