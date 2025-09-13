use crate::{
    HandlebarsContext, SharedState,
    authenticated::UserExtension,
    errors::AppResponse,
    models::account::Account,
    utilities::responses::{ResponseFormat, generate_response, get_response_format},
};
use axum::{
    Extension, Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use handlebars::to_json;
use tokio_postgres::GenericClient;

pub async fn action(
    shared_state: State<SharedState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
    user: Extension<UserExtension>,
    Extension(context): Extension<HandlebarsContext>,
) -> AppResponse {
    let account = Account::get_one(shared_state.pool.get().await?.client(), id, user.id).await?;
    let mut context = context.clone();
    let response_format = get_response_format(&headers)?;
    context.insert("id".to_string(), to_json(account.id));
    context.insert("name".to_string(), to_json(&account.name));
    context.insert("debt".to_string(), to_json(account.debt));
    context.insert("amount".to_string(), to_json(account.amount));
    match response_format {
        ResponseFormat::Html => {
            context.insert("partial".to_string(), to_json("accounts/edit"));

            Ok(generate_response(
                &response_format,
                shared_state.handlebars.render("layout", &context)?,
                StatusCode::OK,
            ))
        }
        ResponseFormat::Turbo => Ok(StatusCode::BAD_REQUEST.into_response()),
        ResponseFormat::Json => Ok(generate_response(
            &response_format,
            Json(account),
            StatusCode::OK,
        )),
    }
}
