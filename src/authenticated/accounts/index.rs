use crate::{
    SharedState,
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
use tera::Context;
use tokio_postgres::GenericClient;

pub async fn action(
    shared_state: State<SharedState>,
    headers: HeaderMap,
    user: Extension<UserExtension>,
    Extension(mut context): Extension<Context>,
) -> AppResponse {
    let accounts = Account::get_all(shared_state.pool.get().await?.client(), user.id).await?;
    let response_format = get_response_format(&headers)?;

    match response_format {
        ResponseFormat::Html | ResponseFormat::Turbo => {
            context.insert("accounts", &accounts);
            Ok(generate_response(
                &ResponseFormat::Html,
                shared_state.tera.render("accounts/index.html", &context)?,
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
