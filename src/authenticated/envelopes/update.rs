use super::{EnvelopeForm, schema};
use crate::{
    SharedState, authenticated::UserExtension, errors::AppResponse, models::envelope::Envelope,
    utilities::turbo,
};
use anyhow::anyhow;
use axum::{
    Extension, Form,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Redirect},
};
use rust_decimal::{Decimal, prelude::FromPrimitive};
use tera::Context;
use tokio_postgres::GenericClient;

pub async fn action(
    shared_state: State<SharedState>,
    Extension(user): Extension<UserExtension>,
    Path(id): Path<i32>,
    headers: HeaderMap,
    Form(form): Form<EnvelopeForm>,
) -> AppResponse {
    let json = serde_json::to_value(&form)?;
    let valid = jsonschema::validate(&schema(), &json);
    let is_turbo = turbo::is_turbo_request(&headers)?;

    match valid {
        Ok(_) => {}
        Err(validation_errors) => {
            let mut context = Context::new();

            context.insert("errors", &validation_errors.to_string());
            context.insert("id", &id);
            context.insert("name", &form.name);
            context.insert("amount", &form.amount);

            let template_name = turbo::get_template_name(is_turbo, "envelopes", "form");
            let content = shared_state.tera.render(&template_name, &context)?;

            return Ok(turbo::form_error_response(
                is_turbo,
                content,
                StatusCode::UNPROCESSABLE_ENTITY,
            ));
        }
    }

    let mut envelope =
        Envelope::get_one(shared_state.pool.get().await?.client(), id, user.id).await?;

    envelope.name = form.name.clone();
    envelope.amount = Decimal::from_f64(form.amount)
        .ok_or(anyhow!("could not parse decimal").context("envelopes:update"))?;
    envelope
        .update(shared_state.pool.get().await?.client())
        .await?;

    Ok(Redirect::to("/envelopes").into_response())
}

#[cfg(test)]
mod tests {
    use crate::{models::envelope::Envelope, test_utils::state_for_tests};
    use axum::http::{Method, Request, StatusCode};
    use rust_decimal::Decimal;
    use tokio_postgres::GenericClient;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_update_envelope() {
        let (shared_state, user_extension) = state_for_tests().await.unwrap();
        let user_id = user_extension.0.id;
        let envelope = Envelope {
            id: None,
            name: "envelope".to_string(),
            user_id,
            amount: Decimal::new(1, 0),
        };

        let envelope = envelope
            .create(shared_state.pool.get().await.unwrap().client())
            .await
            .unwrap();

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
            .layer(user_extension);

        let response = app.oneshot(request).await.unwrap();

        // Assert the response
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers().get("location").unwrap(), "/envelopes");

        let envelope = Envelope::get_one(
            shared_state.pool.get().await.unwrap().client(),
            envelope.id.unwrap(),
            user_id,
        )
        .await
        .unwrap();

        assert_eq!(envelope.name, "Updated Envelope");
        assert_eq!(envelope.amount, Decimal::new(200, 0));
    }
}
