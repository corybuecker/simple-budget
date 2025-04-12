use crate::{
    SharedState,
    authenticated::UserExtension,
    errors::AppResponse,
    models::envelope::Envelope,
    utilities::responses::{
        ResponseFormat, generate_response, get_response_format, get_template_name,
    },
};
use axum::{
    Extension, Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use tera::Context;
use tokio_postgres::GenericClient;

pub async fn modal(
    shared_state: State<SharedState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
    Extension(user): Extension<UserExtension>,
    Extension(mut context): Extension<Context>,
) -> AppResponse {
    let envelope = Envelope::get_one(shared_state.pool.get().await?.client(), id, user.id).await?;
    let response_format = get_response_format(&headers)?;

    match response_format {
        ResponseFormat::Html => {
            context.insert("envelope", &envelope);
            Ok(generate_response(
                &response_format,
                shared_state
                    .tera
                    .render("envelopes/delete/confirm.html", &context)?,
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
    Extension(mut context): Extension<Context>,
) -> AppResponse {
    let envelope = Envelope::get_one(shared_state.pool.get().await?.client(), id, user.id).await?;
    envelope
        .delete(shared_state.pool.get().await?.client())
        .await?;
    let response_format = get_response_format(&headers)?;
    let template_name = get_template_name(&response_format, "envelopes", "delete");

    match response_format {
        ResponseFormat::Html => Ok(StatusCode::NOT_ACCEPTABLE.into_response()),
        ResponseFormat::Json => Ok(generate_response(
            &response_format,
            Json(envelope),
            StatusCode::OK,
        )),
        ResponseFormat::Turbo => {
            context.insert("envelope", &envelope);

            Ok(generate_response(
                &response_format,
                shared_state.tera.render(&template_name, &context)?,
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
    use tokio_postgres::GenericClient;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_delete_modal() {
        let (shared_state, user_extension, context_extension) = state_for_tests().await.unwrap();
        let envelope = Envelope {
            id: None,
            user_id: user_extension.0.id,
            name: "Test Envelope".to_string(),
            amount: Decimal::new(100, 0),
        };

        let envelope = envelope
            .create(shared_state.pool.get().await.unwrap().client())
            .await
            .unwrap();

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

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_delete_action() {
        let (shared_state, user_extension, context_extension) = state_for_tests().await.unwrap();
        let user_id = user_extension.0.id;
        let envelope = Envelope {
            id: None,
            user_id: user_extension.0.id,
            name: "Test Envelope".to_string(),
            amount: Decimal::new(100, 0),
        };

        let envelope = envelope
            .create(shared_state.pool.get().await.unwrap().client())
            .await
            .unwrap();

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

        let deleted_envelope = Envelope::get_one(
            shared_state.pool.get().await.unwrap().client(),
            envelope.id.unwrap(),
            user_id,
        )
        .await;
        assert!(deleted_envelope.is_err());
    }
}
