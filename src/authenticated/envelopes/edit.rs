use crate::{
    HandlebarsContext, SharedState,
    authenticated::UserExtension,
    errors::AppResponse,
    models::envelope::Envelope,
    utilities::responses::{ResponseFormat, generate_response, get_response_format},
};
use axum::{
    Extension, Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
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
    let envelope = Envelope::get_one(shared_state.pool.get().await?.client(), id, user.id).await?;
    let mut context = context.clone();
    let response_format = get_response_format(&headers)?;

    match response_format {
        ResponseFormat::Html => {
            context.insert("partial".to_string(), to_json("envelopes/edit"));
            context.insert("envelope".to_string(), to_json(&envelope));

            Ok(generate_response(
                &response_format,
                shared_state.handlebars.render("layout", &context)?,
                StatusCode::OK,
            ))
        }
        ResponseFormat::Turbo => {
            context.insert("envelope".to_string(), to_json(&envelope));

            Ok(generate_response(
                &response_format,
                shared_state.handlebars.render("envelopes/edit", &context)?,
                StatusCode::OK,
            ))
        }
        ResponseFormat::Json => Ok(generate_response(
            &response_format,
            Json(envelope),
            StatusCode::OK,
        )),
    }
}
