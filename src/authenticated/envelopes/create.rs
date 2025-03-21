use super::{EnvelopeForm, schema};
use crate::{
    SharedState, authenticated::UserExtension, errors::AppResponse, models::envelope::Envelope,
};
use axum::{
    Extension, Form,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect},
};
use rust_decimal::{Decimal, prelude::FromPrimitive};
use tera::Context;
use tokio_postgres::GenericClient;

pub async fn page(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
    headers: HeaderMap,
    Form(form): Form<EnvelopeForm>,
) -> AppResponse {
    let json = serde_json::to_value(&form)?;
    let valid = jsonschema::validate(&schema(), &json);

    let mut context = Context::new();

    let mut turbo = false;
    let accept = headers.get("Accept");
    if let Some(accept) = accept {
        if accept.to_str().unwrap().contains("turbo") {
            turbo = true;
        }
    }
    match valid {
        Ok(_) => {}
        Err(validation_errors) => {
            context.insert("errors", &validation_errors.to_string());
            context.insert("name", &form.name);
            context.insert("amount", &form.amount);

            let content = shared_state.tera.render(
                if turbo {
                    "envelopes/form.turbo.html"
                } else {
                    "envelopes/new.html"
                },
                &context,
            )?;

            if turbo {
                return Ok((
                    StatusCode::BAD_REQUEST,
                    [("content-type", "text/vnd.turbo-stream.html")],
                    Html::from(content),
                )
                    .into_response());
            } else {
                return Ok((StatusCode::BAD_REQUEST, Html::from(content)).into_response());
            }
        }
    }

    let envelope = Envelope {
        id: None,
        name: form.name.to_owned(),
        amount: Decimal::from_f64(form.amount.to_owned()).expect("could not parse decimal"),
        user_id: user.id,
    };

    envelope
        .create(shared_state.pool.get().await?.client())
        .await?;

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
    use std::str::from_utf8;
    use tokio_postgres::GenericClient;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_create_envelope_success() {
        let (shared_state, user_extension) = state_for_tests().await.unwrap();
        let client = shared_state.pool.get().await.unwrap();
        let client = client.client();
        let user_id = user_extension.0.id;

        let app = Router::new()
            .route("/envelopes/create", post(page))
            .with_state(shared_state.clone())
            .layer(user_extension);

        let form_data = "name=test_create_envelope_success&amount=300".to_string();
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
        let (shared_state, user_extension) = state_for_tests().await.unwrap();

        let app = Router::new()
            .route("/envelopes/create", post(page))
            .with_state(shared_state)
            .layer(user_extension);

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
        assert!(body_str.contains("test"))
    }

    #[tokio::test]
    async fn test_create_envelope_turbo_stream() {
        let (shared_state, user_extension) = state_for_tests().await.unwrap();

        let app = Router::new()
            .route("/envelopes/create", post(page))
            .with_state(shared_state)
            .layer(user_extension);

        let form_data = "name=t&amount=3400";
        let request = Request::builder()
            .method("POST")
            .uri("/envelopes/create")
            .header("content-type", "application/x-www-form-urlencoded")
            .header("Accept", "text/vnd.turbo-stream.html")
            .body(Body::from(form_data))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "text/vnd.turbo-stream.html"
        );
    }
}
