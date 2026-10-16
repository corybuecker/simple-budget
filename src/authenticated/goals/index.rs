use super::{GoalRow, goal_header_for};
use crate::{
    LayoutCtx, SharedState,
    authenticated::UserExtension,
    errors::AppResponse,
    models::{
        goal::Goal,
        user::{User, preferences::GoalHeader},
    },
    utilities::responses::generate_response,
};
use askama::Template;
use axum::{Extension, extract::State, http::StatusCode};

#[derive(Template)]
#[template(path = "goals/index.html")]
struct Index {
    layout: LayoutCtx,
    goal_header: GoalHeader,
    rows: Vec<GoalRow>,
}

pub async fn action(
    shared_state: State<SharedState>,
    Extension(user): Extension<UserExtension>,
    Extension(layout): Extension<LayoutCtx>,
) -> AppResponse {
    let client = shared_state.pool.get_client().await?;
    let user = User::get_by_id(&client, user.id).await?;
    let goals = Goal::get_all(&client, user.id).await?;

    let template = Index {
        layout,
        goal_header: goal_header_for(&user),
        rows: GoalRow::from_goals(goals)?,
    };

    Ok(generate_response(template.render()?, StatusCode::OK))
}
