use crate::{
    SharedState,
    authenticated::UserExtension,
    errors::AppResponse,
    models::user::{Preferences, User},
};
use axum::{
    Extension,
    extract::State,
    response::{Html, IntoResponse},
};
use postgres_types::Json;
use tera::{Context, Tera};
use tokio_postgres::GenericClient;

pub async fn action(
    state: State<SharedState>,
    user: Extension<UserExtension>,
    Extension(mut context): Extension<Context>,
) -> AppResponse {
    let user = User::get_by_id(state.pool.get().await?.client(), user.id).await?;
    let preferences = user.preferences.unwrap_or(Json(Preferences::default())).0;

    context.insert("timezone", &preferences.timezone);
    context.insert("monthly_income", &preferences.monthly_income);

    let tera: &Tera = &state.tera;
    let content = tera.render("preferences/index.html", &context)?;

    Ok(Html::from(content).into_response())
}
