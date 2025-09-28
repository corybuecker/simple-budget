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
    response::IntoResponse,
};
use handlebars::to_json;

pub async fn modal(
    shared_state: State<SharedState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
    Extension(user): Extension<UserExtension>,
    Extension(context): Extension<HandlebarsContext>,
) -> AppResponse {
    let client = shared_state.pool.get_client().await?;
    let envelope = Envelope::get_one(&client, id, user.id).await?;
    let response_format = get_response_format(&headers)?;

    match response_format {
        ResponseFormat::Html => {
            let mut context = context.clone();
            context.insert(
                "prompt".to_string(),
                to_json("Are you sure you want to delete this envelope?"),
            );
            context.insert("action".to_string(), to_json(format!("/envelopes/{}", id)));
            context.insert("entity".to_string(), to_json(envelope.name));
            context.insert("partial".to_string(), to_json("delete_confirmation"));
            Ok(generate_response(
                &response_format,
                shared_state
                    .handlebars
                    .render("delete_confirmation", &context)?,
                StatusCode::OK,
            ))
        }
        ResponseFormat::Turbo => Ok(StatusCode::NOT_ACCEPTABLE.into_response()),
        ResponseFormat::Json => Ok(generate_response(
            &response_format,
            Json(envelope),
            StatusCode::OK,
        )),
    }
}

pub async fn action(
    shared_state: State<SharedState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
    Extension(user): Extension<UserExtension>,
    Extension(context): Extension<HandlebarsContext>,
) -> AppResponse {
    let client = shared_state.pool.get_client().await?;
    let envelope = Envelope::get_one(&client, id, user.id).await?;
    envelope.delete(&client).await?;
    let response_format = get_response_format(&headers)?;

    match response_format {
        ResponseFormat::Html => Ok(StatusCode::NOT_ACCEPTABLE.into_response()),
        ResponseFormat::Json => Ok(generate_response(
            &response_format,
            Json(envelope),
            StatusCode::OK,
        )),
        ResponseFormat::Turbo => {
            let mut context = context.clone();
            context.insert("envelope".to_string(), to_json(&envelope));

            Ok(generate_response(
                &response_format,
                shared_state
                    .handlebars
                    .render("envelopes/delete", &context)?,
                StatusCode::OK,
            ))
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::models::envelope::Envelope;
    use crate::test_utils::state_for_tests;
    use axum::Router;
    use axum::body::Body;
    use axum::http::Request;
    use rust_decimal::Decimal;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_delete_modal() {
        let (shared_state, user_extension, context_extension) = state_for_tests().await.unwrap();
        let client = shared_state.pool.get_client().await.unwrap();

        let envelope = Envelope {
            id: None,
            user_id: user_extension.0.id,
            name: "Test Envelope".to_string(),
            amount: Decimal::new(100, 0),
        };

        let envelope = envelope.create(&client).await.unwrap();

        let app = Router::new()
            .route("/envelopes/{id}/delete", axum::routing::get(modal))
            .layer(user_extension)
            .layer(context_extension)
            .with_state(shared_state.clone());

        let request = Request::builder()
            .uri(format!("/envelopes/{}/delete", envelope.id.unwrap()))
            .method("GET")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        let (parts, _body) = response.into_parts();

        assert_eq!(parts.status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_delete_action() {
        let (shared_state, user_extension, context_extension) = state_for_tests().await.unwrap();
        let client = shared_state.pool.get_client().await.unwrap();
        let user_id = user_extension.0.id;
        let envelope = Envelope {
            id: None,
            user_id: user_extension.0.id,
            name: "Test Envelope".to_string(),
            amount: Decimal::new(100, 0),
        };

        let envelope = envelope.create(&client).await.unwrap();

        let app = Router::new()
            .route("/envelopes/{id}", axum::routing::delete(action))
            .layer(user_extension)
            .layer(context_extension)
            .with_state(shared_state.clone());

        let request = Request::builder()
            .uri(format!("/envelopes/{}", envelope.id.unwrap()))
            .method("DELETE")
            .header("Accept", "turbo")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let deleted_envelope = Envelope::get_one(&client, envelope.id.unwrap(), user_id).await;
        assert!(deleted_envelope.is_err());
    }
}
