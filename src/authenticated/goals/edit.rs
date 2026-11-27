use super::EditTemplate;
use crate::{
    LayoutCtx, SharedState, authenticated::UserExtension, errors::AppResponse, models::goal::Goal,
    utilities::responses::generate_response,
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
    let goal = Goal::get_one(&client, id, user.id).await?;

    let template = EditTemplate {
        layout,
        id,
        name: goal.name,
        target: goal.target.to_string(),
        target_date: goal.target_date.format("%Y-%m-%d").to_string(),
        recurrence: format!("{:?}", goal.recurrence).to_lowercase(),
        errors: String::new(),
    };

    Ok(generate_response(template.render()?, StatusCode::OK))
}
