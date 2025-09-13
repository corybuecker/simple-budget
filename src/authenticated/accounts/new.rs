use crate::{
    HandlebarsContext, SharedState,
    errors::AppResponse,
    utilities::responses::{ResponseFormat, generate_response, get_response_format},
};
use axum::{
    Extension,
    extract::State,
    http::{HeaderMap, StatusCode},
};
use handlebars::to_json;
// use tracing::debug;

pub async fn action(
    shared_state: State<SharedState>,
    headers: HeaderMap,
    Extension(context): Extension<HandlebarsContext>,
) -> AppResponse {
    let mut context = context.clone();
    context.insert("name".to_string(), to_json(""));
    context.insert("amount".to_string(), to_json(""));
    context.insert("debt".to_string(), to_json(""));

    let response_format = get_response_format(&headers)?;

    match response_format {
        ResponseFormat::Html => {
            context.insert("partial".to_string(), to_json("accounts/new"));

            Ok(generate_response(
                &response_format,
                shared_state.handlebars.render("layout", &context)?,
                StatusCode::OK,
            ))
        }
        ResponseFormat::Turbo => Ok(generate_response(
            &response_format,
            shared_state.handlebars.render("accounts/new", &context)?,
            StatusCode::OK,
        )),
        ResponseFormat::Json => Ok(generate_response(&response_format, "{}", StatusCode::OK)),
    }
}
