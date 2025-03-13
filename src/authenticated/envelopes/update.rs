use super::EnvelopeForm;
use crate::{
    SharedState, authenticated::UserExtension, errors::AppResponse, models::envelope::Envelope,
};
use axum::{
    Extension, Form,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect},
};
use tera::Context;
use validator::Validate;

pub async fn action(
    shared_state: State<SharedState>,
    Extension(user): Extension<UserExtension>,
    Path(id): Path<i32>,
    headers: HeaderMap,
    form: Form<EnvelopeForm>,
) -> AppResponse {
    let mut turbo = false;
    let accept = headers.get("Accept");
    if let Some(accept) = accept {
        if accept.to_str().unwrap().contains("turbo") {
            turbo = true;
        }
    }

    match form.validate() {
        Ok(_) => {}
        Err(validation_errors) => {
            let mut context = Context::new();

            context.insert("errors", &validation_errors.to_string());
            context.insert("id", &id);
            context.insert("name", &form.name);
            context.insert("amount", &form.amount);

            let content = shared_state.tera.render(
                if turbo {
                    "envelopes/form.turbo.html"
                } else {
                    "envelopes/edit.html"
                },
                &context,
            )?;

            if turbo {
                return Ok((
                    StatusCode::UNPROCESSABLE_ENTITY,
                    [("content-type", "text/vnd.turbo-stream.html")],
                    Html::from(content),
                )
                    .into_response());
            } else {
                return Ok((StatusCode::UNPROCESSABLE_ENTITY, Html::from(content)).into_response());
            }
        }
    }

    let mut envelope = Envelope::get_one(&shared_state.client, id, user.id).await?;

    envelope.name = form.name.clone();
    envelope.amount = form.amount;
    envelope.update(&shared_state.client).await?;

    Ok(Redirect::to("/envelopes").into_response())
}

#[cfg(test)]
mod tests {
    use crate::{models::envelope::Envelope, test_utils::state_for_tests};
    use axum::http::{Method, Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_update_envelope() {
        let (shared_state, user_extension) = state_for_tests().await.unwrap();
        let user_id = user_extension.0.id;
        let envelope = Envelope {
            id: None,
            name: "envelope".to_string(),
            user_id: Some(user_id),
            amount: 1.0,
        };

        let envelope = envelope.create(&shared_state.client).await.unwrap();

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

        let envelope = Envelope::get_one(&shared_state.client, envelope.id.unwrap(), user_id)
            .await
            .unwrap();

        assert_eq!(envelope.name, "Updated Envelope");
        assert_eq!(envelope.amount, 200.0);
    }
}
