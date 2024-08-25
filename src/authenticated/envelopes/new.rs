use crate::{authenticated::UserExtension, SharedState};
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Extension,
};
use tera::Context;

pub async fn page(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
) -> Result<Response, StatusCode> {
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
