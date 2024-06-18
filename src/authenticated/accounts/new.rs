use std::str::FromStr;

use crate::{authenticated::UserExtension, SharedState};
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Extension,
};
use mongodb::{
    bson::{doc, oid::ObjectId, Bson},
    Collection,
};
use serde::{Deserialize, Serialize};
use tera::Context;

#[derive(Serialize, Deserialize)]
struct Account {
    name: String,
    amount: f64,
}

pub async fn page(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
) -> Result<Response, StatusCode> {
    log::debug!("{:?}", user);

    let mut context = Context::new();
    let Ok(content) = shared_state.tera.render("accounts/new.html", &context) else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    Ok(Html::from(content).into_response())
}
