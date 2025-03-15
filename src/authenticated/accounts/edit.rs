use crate::errors::AppResponse;
use crate::models::account::Account;
use crate::{SharedState, authenticated::UserExtension};
use axum::extract::Path;
use axum::{
    Extension,
    extract::State,
    response::{Html, IntoResponse},
};
use tera::Context;

pub async fn page(
    shared_state: State<SharedState>,
    Path(id): Path<i32>,
    user: Extension<UserExtension>,
    Extension(mut context): Extension<Context>,
) -> AppResponse {
    let account = Account::get_one(&shared_state.client, id, user.id).await?;

    context.insert("id", &account.id);
    context.insert("name", &account.name);
    context.insert("amount", &account.amount);
    context.insert("debt", &account.debt);

    let content = shared_state.tera.render("accounts/edit.html", &context)?;

    Ok(Html::from(content).into_response())
}
