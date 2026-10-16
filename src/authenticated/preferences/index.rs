use crate::{
    LayoutCtx, SharedState,
    authenticated::UserExtension,
    errors::AppResponse,
    models::user::{User, preferences::Preferences},
    utilities::responses::generate_response,
};
use askama::Template;
use axum::{Extension, extract::State, http::StatusCode};
use postgres_types::Json as PgJson;

#[derive(Template)]
#[template(path = "preferences/index.html")]
struct Index {
    layout: LayoutCtx,
    timezone: String,
    monthly_income: String,
    accelerate_goals: bool,
    accelerate_non_monthly: bool,
}

pub async fn action(
    shared_state: State<SharedState>,
    Extension(user): Extension<UserExtension>,
    Extension(layout): Extension<LayoutCtx>,
) -> AppResponse {
    let client = shared_state.pool.get_client().await?;
    let user = User::get_by_id(&client, user.id).await?;
    let preferences = user.preferences.unwrap_or(PgJson(Preferences::default())).0;

    let template = Index {
        layout,
        timezone: preferences.timezone.unwrap_or_default(),
        monthly_income: preferences
            .monthly_income
            .map(|income| income.to_string())
            .unwrap_or_default(),
        accelerate_goals: preferences.accelerate_goals.unwrap_or(false),
        accelerate_non_monthly: preferences.accelerate_non_monthly.unwrap_or(false),
    };

    Ok(generate_response(template.render()?, StatusCode::OK))
}
