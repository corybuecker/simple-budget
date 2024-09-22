use crate::SharedState;
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Extension,
};
use tera::Context;

pub async fn page(
    shared_state: State<SharedState>,
    Extension(context): Extension<Context>,
) -> Response {
    let Ok(content) = shared_state.tera.render("goals/new.html", &context) else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    Html::from(content).into_response()
}
