use crate::{authenticated::UserExtension, SharedState};
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Extension,
};
use serde::{Deserialize, Serialize};
use tera::Context;
use validator::Validate;

#[derive(Validate, Serialize, Deserialize)]
struct Envelope {
    #[validate(length(min = 1))]
    name: String,

    #[validate(range(min = 0.0))]
    amount: f64,
    debt: bool,
}

pub async fn page(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
) -> Result<Response, StatusCode> {
    log::debug!("{:?}", user);

    let mut context = Context::new();
    context.insert("csrf", &user.csrf);
    match shared_state.tera.render("envelopes/new.html", &context) {
        Ok(content) => Ok(Html::from(content).into_response()),
        Err(error) => {
            log::error!("{}", error);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
