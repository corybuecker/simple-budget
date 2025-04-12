use crate::{
    SharedState,
    authenticated::UserExtension,
    errors::AppResponse,
    models::user::{Preferences, User},
    utilities::responses::{ResponseFormat, generate_response, get_response_format},
};
use axum::{
    Extension, Json,
    extract::State,
    http::{HeaderMap, StatusCode},
};
use postgres_types::Json as PgJson;
use tera::Context;
use tokio_postgres::GenericClient;

pub async fn action(
    state: State<SharedState>,
    headers: HeaderMap,
    user: Extension<UserExtension>,
    Extension(mut context): Extension<Context>,
) -> AppResponse {
    let user = User::get_by_id(state.pool.get().await?.client(), user.id).await?;
    let preferences = user.preferences.unwrap_or(PgJson(Preferences::default())).0;
    let response_format = get_response_format(&headers)?;

    context.insert("timezone", &preferences.timezone);
    context.insert("monthly_income", &preferences.monthly_income);

    match response_format {
        ResponseFormat::Turbo | ResponseFormat::Html => Ok(generate_response(
            &ResponseFormat::Html,
            state.tera.render("preferences/index.html", &context)?,
            StatusCode::OK,
        )),
        ResponseFormat::Json => Ok(generate_response(
            &response_format,
            Json(preferences),
            StatusCode::OK,
        )),
    }
}
