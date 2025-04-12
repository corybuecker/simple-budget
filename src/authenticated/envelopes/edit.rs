use crate::errors::AppResponse;
use crate::models::envelope::Envelope;
use crate::utilities::responses::{
    ResponseFormat, generate_response, get_response_format, get_template_name,
};
use crate::{SharedState, authenticated::UserExtension};
use axum::Json;
use axum::extract::Path;
use axum::http::{HeaderMap, StatusCode};
use axum::{Extension, extract::State};
use tera::Context;
use tokio_postgres::GenericClient;

pub async fn action(
    shared_state: State<SharedState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
    user: Extension<UserExtension>,
    Extension(mut context): Extension<Context>,
) -> AppResponse {
    let envelope = Envelope::get_one(shared_state.pool.get().await?.client(), id, user.id).await?;
    let response_format = get_response_format(&headers)?;
    let template_name = get_template_name(&response_format, "envelopes", "edit");

    match response_format {
        ResponseFormat::Json => Ok(generate_response(
            &response_format,
            Json(envelope),
            StatusCode::OK,
        )),
        ResponseFormat::Html | ResponseFormat::Turbo => {
            context.insert("envelope", &envelope);

            Ok(generate_response(
                &response_format,
                shared_state.tera.render(&template_name, &context)?,
                StatusCode::OK,
            ))
        }
    }
}
