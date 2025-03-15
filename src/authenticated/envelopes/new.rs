use crate::{SharedState, errors::AppResponse};
use axum::{
    Extension,
    extract::State,
    response::{Html, IntoResponse},
};
use tera::Context;

pub async fn page(
    shared_state: State<SharedState>,
    Extension(context): Extension<Context>,
) -> AppResponse {
    let content = shared_state.tera.render("envelopes/new.html", &context)?;

    Ok(Html::from(content).into_response())
}
