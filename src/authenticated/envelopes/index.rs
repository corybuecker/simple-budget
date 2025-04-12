use crate::{
    SharedState,
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
use tera::Context;
use tokio_postgres::GenericClient;

pub async fn action(
    shared_state: State<SharedState>,
    headers: HeaderMap,
    user: Extension<UserExtension>,
    Extension(mut context): Extension<Context>,
) -> AppResponse {
    let envelopes = Envelope::get_all(shared_state.pool.get().await?.client(), user.id).await?;
    let response_format = get_response_format(&headers)?;

    match response_format {
        ResponseFormat::Turbo | ResponseFormat::Html => {
            context.insert("envelopes", &envelopes);
            Ok(generate_response(
                &ResponseFormat::Html,
                shared_state.tera.render("envelopes/index.html", &context)?,
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
