use super::{EditTemplate, EnvelopeForm, schema};
use crate::{
    LayoutCtx, SharedState, authenticated::UserExtension, errors::AppResponse,
    models::envelope::Envelope, utilities::responses::generate_response,
};
use anyhow::anyhow;
use askama::Template;
use axum::{
    Extension, Form,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use rust_decimal::{Decimal, prelude::FromPrimitive};
use uuid::Uuid;

pub async fn action(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
    Path(id): Path<Uuid>,
    Extension(layout): Extension<LayoutCtx>,
    Form(form): Form<EnvelopeForm>,
) -> AppResponse {
    let client = shared_state.pool.get_client().await?;
    let json = serde_json::to_value(&form)?;

    if let Err(validation_errors) = jsonschema::validate(&schema(), &json) {
        let template = EditTemplate {
            layout,
            id,
            name: form.name,
            amount: form.amount.to_string(),
            errors: validation_errors.to_string(),
        };
        return Ok(generate_response(
            template.render()?,
            StatusCode::BAD_REQUEST,
        ));
    }

    let mut envelope = Envelope::get_one(&client, id, user.id).await?;
    envelope.name = form.name.clone();
    envelope.amount =
        Decimal::from_f64(form.amount).ok_or_else(|| anyhow!("could not parse decimal"))?;
    envelope.update(&client).await?;

    Ok(Redirect::to("/envelopes").into_response())
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
