use super::{EnvelopeForm, NewTemplate, schema};
use crate::{
    LayoutCtx, SharedState, authenticated::UserExtension, errors::AppResponse,
    models::envelope::Envelope, utilities::responses::generate_response,
};
use anyhow::anyhow;
use askama::Template;
use axum::{
    Extension, Form,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;

pub async fn action(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
    Extension(layout): Extension<LayoutCtx>,
    Form(form): Form<EnvelopeForm>,
) -> AppResponse {
    let json = serde_json::to_value(&form)?;

    if let Err(validation_errors) = jsonschema::validate(&schema(), &json) {
        let template = NewTemplate {
            layout,
            name: form.name,
            amount: form.amount.to_string(),
            errors: validation_errors.to_string(),
        };
        return Ok(generate_response(
            template.render()?,
            StatusCode::BAD_REQUEST,
        ));
    }

    let envelope = Envelope {
        id: None,
        name: form.name.to_owned(),
        amount: Decimal::from_f64(form.amount.to_owned())
            .ok_or_else(|| anyhow!("could not parse decimal"))?,
        user_id: user.id,
    };
    let client = &shared_state.pool.get_client().await?;
    envelope.create(client).await?;

    Ok(Redirect::to("/envelopes").into_response())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::state_for_tests;
    use axum::Router;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use axum::routing::post;
    use rust_database_common::GenericClient;
    use std::str::from_utf8;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_create_envelope_success() {
        let (shared_state, user_extension, context_extension) = state_for_tests().await.unwrap();
        let user_id = user_extension.0.id;

        let app = Router::new()
            .route("/envelopes/create", post(action))
            .with_state(shared_state.clone())
            .layer(user_extension)
            .layer(context_extension);
        let client = &shared_state.pool.get_client().await.unwrap();

        let form_data = "name=test_create_envelope_success&amount=300.00";
        let request = Request::builder()
            .method("POST")
            .uri("/envelopes/create")
            .header("content-type", "application/x-www-form-urlencoded")
            .body(Body::from(form_data))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers().get("location").unwrap(), "/envelopes");

        let envelope = client
            .query_one(
                "SELECT * FROM envelopes WHERE user_id = $1 LIMIT 1",
                &[&user_id],
            )
            .await
            .unwrap();

        assert_eq!(
            envelope.get::<_, String>("name"),
            "test_create_envelope_success"
        )
    }

    #[tokio::test]
    async fn test_create_envelope_validation_error() {
        let (shared_state, user_extension, context_extension) = state_for_tests().await.unwrap();

        let app = Router::new()
            .route("/envelopes/create", post(action))
            .with_state(shared_state)
            .layer(user_extension)
            .layer(context_extension);

        let form_data = "name=t&amount=300";
        let request = Request::builder()
            .method("POST")
            .uri("/envelopes/create")
            .header("content-type", "application/x-www-form-urlencoded")
            .body(Body::from(form_data))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        let (parts, body) = response.into_parts();
        let bytes = to_bytes(body, usize::MAX).await.unwrap();
        let body_str = from_utf8(&bytes).unwrap().to_string();
        assert_eq!(parts.status, StatusCode::BAD_REQUEST);
        assert!(body_str.contains(r#"value="300""#))
    }
}
