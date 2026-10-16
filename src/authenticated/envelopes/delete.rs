use crate::{
    LayoutCtx, SharedState, authenticated::UserExtension, errors::AppResponse,
    models::envelope::Envelope, templates::DeleteConfirmation,
    utilities::responses::generate_response,
};
use askama::Template;
use axum::{
    Extension,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use uuid::Uuid;

pub async fn modal(
    shared_state: State<SharedState>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<UserExtension>,
    Extension(layout): Extension<LayoutCtx>,
) -> AppResponse {
    let client = shared_state.pool.get_client().await?;
    let envelope = Envelope::get_one(&client, id, user.id).await?;

    let template = DeleteConfirmation {
        layout,
        prompt: "Are you sure you want to delete this envelope?".to_string(),
        action: format!("/envelopes/{}/delete", id),
        cancel: "/envelopes".to_string(),
        entity: envelope.name,
    };

    Ok(generate_response(template.render()?, StatusCode::OK))
}

pub async fn action(
    shared_state: State<SharedState>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<UserExtension>,
) -> AppResponse {
    let client = shared_state.pool.get_client().await?;
    let envelope = Envelope::get_one(&client, id, user.id).await?;

    envelope.delete(&client).await?;

    Ok(Redirect::to("/envelopes").into_response())
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
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers().get("location").unwrap(), "/envelopes");

        let deleted_envelope = Envelope::get_one(&client, envelope.id.unwrap(), user_id).await;
        assert!(deleted_envelope.is_err());
    }
}
