use super::{EnvelopeForm, schema};
use crate::{
    HandlebarsContext, SharedState,
    authenticated::UserExtension,
    errors::AppResponse,
    models::envelope::Envelope,
    utilities::responses::{self, generate_response, get_response_format},
};
use anyhow::anyhow;
use axum::{
    Extension, Form, Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Redirect},
};
use handlebars::to_json;
use rust_decimal::{Decimal, prelude::FromPrimitive};

pub async fn action(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
    Path(id): Path<i32>,
    headers: HeaderMap,
    Extension(context): Extension<HandlebarsContext>,
    Form(form): Form<EnvelopeForm>,
) -> AppResponse {
    let json = serde_json::to_value(&form)?;
    let valid = jsonschema::validate(&schema(), &json);
    let response_format = responses::get_response_format(&headers)?;

    match valid {
        Ok(_) => {}
        Err(validation_errors) => {
            let mut context = context.clone();

            context.insert("errors".to_string(), to_json(validation_errors.to_string()));
            context.insert("id".to_string(), to_json(id));
            context.insert("name".to_string(), to_json(&form.name));
            context.insert("amount".to_string(), to_json(form.amount));

            match response_format {
                responses::ResponseFormat::Html => {
                    context.insert("partial".to_string(), to_json("envelopes/form"));
                    return Ok(responses::generate_response(
                        &responses::ResponseFormat::Html,
                        shared_state.handlebars.render("layout", &context)?,
                        StatusCode::BAD_REQUEST,
                    ));
                }
                responses::ResponseFormat::Turbo => {
                    return Ok(responses::generate_response(
                        &response_format,
                        shared_state.handlebars.render("envelopes/form", &context)?,
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
    let client = shared_state.pool.get_client().await?;
    let mut envelope = Envelope::get_one(&client, id, user.id).await?;

    envelope.name = form.name.clone();
    envelope.amount =
        Decimal::from_f64(form.amount).ok_or_else(|| anyhow!("could not parse decimal"))?;
    envelope.update(&client).await?;

    match get_response_format(&headers)? {
        responses::ResponseFormat::Html | responses::ResponseFormat::Turbo => {
            Ok(Redirect::to("/envelopes").into_response())
        }
        responses::ResponseFormat::Json => Ok(generate_response(
            &responses::ResponseFormat::Json,
            Json(envelope),
            StatusCode::OK,
        )),
    }
}

#[cfg(test)]
mod tests {
    use crate::{models::envelope::Envelope, test_utils::state_for_tests};
    use axum::http::{Method, Request, StatusCode};
    use rust_decimal::Decimal;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_update_envelope() {
        let (shared_state, user_extension, context_extension) = state_for_tests().await.unwrap();
        let client = shared_state.pool.get_client().await.unwrap();
        let user_id = user_extension.0.id;
        let envelope = Envelope {
            id: None,
            name: "envelope".to_string(),
            user_id,
            amount: Decimal::new(1, 0),
        };

        let envelope = envelope.create(&client).await.unwrap();

        let request = Request::builder()
            .method(Method::POST)
            .uri(format!("/envelopes/{}", envelope.id.unwrap()))
            .header("content-type", "application/x-www-form-urlencoded")
            .body("name=Updated%20Envelope&amount=200.0".to_string())
            .unwrap();

        // Create a test app and call the action
        let app = axum::Router::new()
            .route(
                "/envelopes/{id}",
                axum::routing::post(crate::authenticated::envelopes::update::action),
            )
            .with_state(shared_state.clone())
            .layer(user_extension)
            .layer(context_extension);

        let response = app.oneshot(request).await.unwrap();

        // Assert the response
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers().get("location").unwrap(), "/envelopes");

        let envelope = Envelope::get_one(&client, envelope.id.unwrap(), user_id)
            .await
            .unwrap();

        assert_eq!(envelope.name, "Updated Envelope");
        assert_eq!(envelope.amount, Decimal::new(200, 0));
    }
}
