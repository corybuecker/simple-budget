use crate::{
    LayoutCtx, SharedState,
    authenticated::UserExtension,
    errors::AppResponse,
    models::account::Account,
    utilities::responses::{ResponseFormat, generate_response},
};
use askama::Template;
use axum::{Extension, extract::State, http::StatusCode};

#[derive(Template)]
#[template(path = "accounts/index.html")]
struct Index {
    #[allow(unused)]
    layout: LayoutCtx,
    accounts: Vec<Account>,
}

pub async fn action(
    shared_state: State<SharedState>,
    Extension(user): Extension<UserExtension>,
    Extension(layout): Extension<LayoutCtx>,
) -> AppResponse {
    let client = shared_state.pool.get_client().await?;
    let accounts = Account::get_all(&client, user.id).await?;
    let template = Index { layout, accounts };

    Ok(generate_response(
        &ResponseFormat::Html,
        template.render()?,
        StatusCode::OK,
    ))
}
