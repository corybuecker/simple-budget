use crate::{authenticated::UserExtension, errors::FormError, SharedState};
use axum::{
    extract::State,
    response::{Html, IntoResponse, Response},
    Extension,
};
use tera::Context;

pub async fn page(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
) -> Result<Response, FormError> {
    let mut context = Context::new();
    context.insert("csrf", &user.csrf);

    let content = shared_state.tera.render("accounts/new.html", &context)?;

    Ok(Html::from(content).into_response())
}
