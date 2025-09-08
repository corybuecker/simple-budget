use crate::{
    HandlebarsContext, SharedState,
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
use handlebars::to_json;
use postgres_types::Json as PgJson;
use tokio_postgres::GenericClient;

pub async fn action(
    state: State<SharedState>,
    headers: HeaderMap,
    user: Extension<UserExtension>,
    Extension(context): Extension<HandlebarsContext>,
) -> AppResponse {
    let user = User::get_by_id(state.pool.get().await?.client(), user.id).await?;
    let preferences = user.preferences.unwrap_or(PgJson(Preferences::default())).0;
    let response_format = get_response_format(&headers)?;
    let mut context = context.clone();

    context.insert("timezone".to_string(), to_json(&preferences.timezone));
    context.insert(
        "monthly_income".to_string(),
        to_json(preferences.monthly_income),
    );
    context.insert("partial".to_string(), to_json("preferences/index"));

    match response_format {
        ResponseFormat::Turbo | ResponseFormat::Html => Ok(generate_response(
            &ResponseFormat::Html,
            state.handlebars.render("layout", &context)?,
            StatusCode::OK,
        )),
        ResponseFormat::Json => Ok(generate_response(
            &response_format,
            Json(preferences),
            StatusCode::OK,
        )),
    }
}
