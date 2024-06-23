use crate::{authenticated::UserExtension, SharedState};
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Extension,
};
use tera::Context;

pub async fn page(shared_state: State<SharedState>, user: Extension<UserExtension>) -> Response {
    log::debug!("{:?}", user);
    let context = Context::new();
    let Ok(content) = shared_state.tera.render("goals/index.html", &context) else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    Html::from(content).into_response()
}
