use crate::{
    SharedState, authenticated::UserExtension, errors::FormError, models::account::Account,
};
use axum::{
    Extension,
    extract::State,
    response::{Html, IntoResponse, Response},
};
use tera::Context;

pub async fn page(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
    Extension(mut context): Extension<Context>,
) -> Result<Response, FormError> {
    let accounts = Account::get_all(&shared_state.client, user.id).await?;
    context.insert("accounts", &accounts);

    let content = shared_state.tera.render("accounts/index.html", &context)?;

    Ok(Html::from(content).into_response())
}
