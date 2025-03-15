use crate::{
    SharedState, authenticated::UserExtension, errors::AppResponse, models::account::Account,
};
use axum::{
    Extension,
    extract::State,
    response::{Html, IntoResponse},
};
use tera::Context;
use tokio_postgres::GenericClient;

pub async fn page(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
    Extension(mut context): Extension<Context>,
) -> AppResponse {
    let accounts = Account::get_all(shared_state.pool.get().await?.client(), user.id).await?;
    context.insert("accounts", &accounts);

    let content = shared_state.tera.render("accounts/index.html", &context)?;

    Ok(Html::from(content).into_response())
}
