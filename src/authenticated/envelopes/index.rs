use crate::{
    HandlebarsContext, SharedState,
    authenticated::UserExtension,
    errors::AppResponse,
    models::envelope::Envelope,
    utilities::responses::{ResponseFormat, generate_response, get_response_format},
};
use axum::{
    Extension, Json,
    extract::State,
    http::{HeaderMap, StatusCode},
};
use handlebars::to_json;

pub async fn action(
    shared_state: State<SharedState>,
    headers: HeaderMap,
    user: Extension<UserExtension>,
    Extension(context): Extension<HandlebarsContext>,
) -> AppResponse {
    let client = shared_state.pool.get_client().await?;
    let mut context = context.clone();
    let envelopes = Envelope::get_all(&client, user.id).await?;
    let response_format = get_response_format(&headers)?;

    match response_format {
        ResponseFormat::Html | ResponseFormat::Turbo => {
            context.insert("envelopes".to_string(), to_json(envelopes));
            context.insert("partial".to_string(), to_json("envelopes/index"));

            Ok(generate_response(
                &ResponseFormat::Html,
                shared_state.handlebars.render("layout", &context)?,
                StatusCode::OK,
            ))
        }
        ResponseFormat::Json => Ok(generate_response(
            &response_format,
            Json(envelopes),
            StatusCode::OK,
        )),
    }
}
