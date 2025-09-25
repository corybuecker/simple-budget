use super::{EnvelopeForm, schema};
use crate::errors::AppResponse;
use crate::{
    HandlebarsContext, SharedState, authenticated::UserExtension, models::envelope::Envelope,
    utilities::responses,
};
use anyhow::anyhow;
use axum::{
    Extension, Form,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Redirect},
};
use handlebars::to_json;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;

pub async fn action(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
    headers: HeaderMap,
    Extension(context): Extension<HandlebarsContext>,
    Form(form): Form<EnvelopeForm>,
) -> AppResponse {
    let client = shared_state.pool.get_client().await?;
    let json = serde_json::to_value(&form)?;
    let valid = jsonschema::validate(&schema(), &json);

    let response_format = responses::get_response_format(&headers)?;

    match valid {
        Ok(_) => {}
        Err(validation_errors) => {
            let mut context = context.clone();

            context.insert("errors".to_string(), to_json(validation_errors.to_string()));
            context.insert("name".to_string(), to_json(&form.name));
            context.insert("amount".to_string(), to_json(form.amount));

            match response_format {
                responses::ResponseFormat::Html => {
                    context.insert("partial".to_string(), to_json("envelopes/new"));
                    return Ok(responses::generate_response(
                        &responses::ResponseFormat::Html,
                        shared_state.handlebars.render("layout", &context)?,
                        StatusCode::BAD_REQUEST,
                    ));
                }
                responses::ResponseFormat::Turbo => {
                    return Ok(responses::generate_response(
                        &response_format,
                        shared_state
                            .handlebars
                            .render("envelopes/_form.turbo", &context)?,
                        StatusCode::BAD_REQUEST,
                    ));
                }
                responses::ResponseFormat::Json => {
                    return Ok(responses::generate_response(
                        &response_format,
                        serde_json::to_string(&context)?,
                        StatusCode::BAD_REQUEST,
                    ));
                }
            }
        }
    }

    let envelope = Envelope {
        id: None,
        name: form.name.to_owned(),
        amount: Decimal::from_f64(form.amount.to_owned())
            .ok_or_else(|| anyhow!("could not parse decimal"))?,
        user_id: user.id,
    };

    envelope.create(&client).await?;

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
        assert!(body_str.contains("300.0"))
    }

    #[tokio::test]
    async fn test_create_envelope_turbo_stream() {
        let (shared_state, user_extension, context_extension) = state_for_tests().await.unwrap();

        let app = Router::new()
            .route("/envelopes/create", post(action))
            .with_state(shared_state)
            .layer(user_extension)
            .layer(context_extension);

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
