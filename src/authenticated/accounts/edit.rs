use super::EditTemplate;
use crate::{
    LayoutCtx, SharedState, authenticated::UserExtension, errors::AppResponse,
    models::account::Account, utilities::responses::generate_response,
};
use askama::Template;
use axum::{
    Extension,
    extract::{Path, State},
    http::StatusCode,
};
use uuid::Uuid;

pub async fn action(
    shared_state: State<SharedState>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<UserExtension>,
    Extension(layout): Extension<LayoutCtx>,
) -> AppResponse {
    let client = shared_state.pool.get_client().await?;
    let account = Account::get_one(&client, id, user.id).await?;

    let template = EditTemplate {
        layout,
        id,
        name: account.name,
        amount: account.amount.to_string(),
        debt: account.debt,
        errors: String::new(),
    };

    Ok(generate_response(template.render()?, StatusCode::OK))
}
