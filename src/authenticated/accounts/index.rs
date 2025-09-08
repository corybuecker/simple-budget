use crate::{
    HandlebarsContext, SharedState,
    authenticated::UserExtension,
    errors::AppResponse,
    models::account::Account,
    utilities::responses::{ResponseFormat, generate_response, get_response_format},
};
use axum::{
    Extension, Json,
    extract::State,
    http::{HeaderMap, StatusCode},
};
use handlebars::to_json;
use tokio_postgres::GenericClient;

pub async fn action(
    shared_state: State<SharedState>,
    headers: HeaderMap,
    user: Extension<UserExtension>,
    Extension(context): Extension<HandlebarsContext>,
) -> AppResponse {
    let mut context = context.clone();
    let accounts = Account::get_all(shared_state.pool.get().await?.client(), user.id).await?;
    let response_format = get_response_format(&headers)?;

    match response_format {
        ResponseFormat::Html | ResponseFormat::Turbo => {
            context.insert("accounts".to_string(), to_json(accounts));
            context.insert("partial".to_string(), to_json("accounts/index"));

            Ok(generate_response(
                &ResponseFormat::Html,
                shared_state.handlebars.render("layout", &context)?,
                StatusCode::OK,
            ))
        }
        ResponseFormat::Json => Ok(generate_response(
            &response_format,
            Json(accounts),
            StatusCode::OK,
        )),
    }
}
