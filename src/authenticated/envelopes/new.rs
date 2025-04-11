use crate::{
    SharedState,
    errors::AppResponse,
    utilities::responses::{generate_response, get_response_format, get_template_name},
};
use axum::{
    Extension,
    extract::State,
    http::{HeaderMap, StatusCode},
};
use tera::Context;

pub async fn action(
    shared_state: State<SharedState>,
    headers: HeaderMap,
    Extension(context): Extension<Context>,
) -> AppResponse {
    let response_format = get_response_format(&headers)?;
    let template = get_template_name(&response_format, "envelopes", "new");
    let content = shared_state.tera.render(&template, &context)?;

    Ok(generate_response(&response_format, content, StatusCode::OK))
}
