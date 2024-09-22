use crate::{errors::FormError, SharedState};
use axum::{
    extract::State,
    response::{Html, IntoResponse, Response},
    Extension,
};
use tera::Context;

pub async fn page(
    shared_state: State<SharedState>,
    Extension(context): Extension<Context>,
) -> Result<Response, FormError> {
    let content = shared_state.tera.render("accounts/new.html", &context)?;

    Ok(Html::from(content).into_response())
}
