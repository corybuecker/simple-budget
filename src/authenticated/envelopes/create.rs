use super::EnvelopeForm;
use crate::{
    SharedState, authenticated::UserExtension, errors::FormError, models::envelope::Envelope,
};
use axum::{
    Extension, Form,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
};
use tera::Context;
use validator::Validate;

pub async fn page(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
    headers: HeaderMap,
    form: Form<EnvelopeForm>,
) -> Result<Response, FormError> {
    let mut context = Context::new();

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
        amount: form.amount.to_owned(),
        user_id: Some(user.id),
    };

    envelope.create(&shared_state.client).await?;

    Ok(Redirect::to("/envelopes").into_response())
}

//#[cfg(test)]
//mod tests {
//    use super::*;
//    use crate::test_utils::{state_for_tests, user_for_tests};
//    use axum::Router;
//    use axum::body::{Body, to_bytes};
//    use axum::http::{Request, StatusCode};
//    use axum::routing::post;
//    use bson::doc;
//    use mongodb::Collection;
//
//    use std::str::from_utf8;
//    use tower::ServiceExt;
//
//    #[tokio::test]
//    async fn test_create_envelope_success() {
//        let shared_state = state_for_tests().await;
//        let envelopes: Collection<Envelope> = shared_state
//            .mongo
//            .default_database()
//            .unwrap()
//            .collection("envelopes");
//
//        envelopes
//            .delete_one(doc! {"name": "test_create_envelope_success"})
//            .await
//            .unwrap();
//
//        let app = Router::new()
//            .route("/envelopes/create", post(page))
//            .with_state(shared_state)
//            .layer(user_for_tests(&ObjectId::new().to_hex()));
//
//        let form_data = "name=test_create_envelope_success&amount=300".to_string();
//        let request = Request::builder()
//            .method("POST")
//            .uri("/envelopes/create")
//            .header("content-type", "application/x-www-form-urlencoded")
//            .body(Body::from(form_data))
//            .unwrap();
//
//        let response = app.oneshot(request).await.unwrap();
//
//        assert_eq!(response.status(), StatusCode::SEE_OTHER);
//        assert_eq!(response.headers().get("location").unwrap(), "/envelopes");
//
//        // Verify that the envelope was created in the database
//        let envelope = envelopes
//            .find_one(doc! {"name": "test_create_envelope_success"})
//            .await
//            .unwrap();
//
//        assert!(envelope.is_some())
//    }
//
//    #[tokio::test]
//    async fn test_create_envelope_validation_error() {
//        let shared_state = state_for_tests().await;
//        let app = Router::new()
//            .route("/envelopes/create", post(page))
//            .with_state(shared_state)
//            .layer(user_for_tests(&ObjectId::new().to_hex()));
//
//        let form_data = "name=test&amount=300";
//        let request = Request::builder()
//            .method("POST")
//            .uri("/envelopes/create")
//            .header("content-type", "application/x-www-form-urlencoded")
//            .body(Body::from(form_data))
//            .unwrap();
//
//        let response = app.oneshot(request).await.unwrap();
//
//        let (parts, body) = response.into_parts();
//        let bytes = to_bytes(body, usize::MAX).await.unwrap();
//        let body_str = from_utf8(&bytes).unwrap().to_string();
//
//        assert_eq!(parts.status, StatusCode::BAD_REQUEST);
//        assert!(body_str.contains("test"))
//    }
//
//    #[tokio::test]
//    async fn test_create_envelope_turbo_stream() {
//        let shared_state = state_for_tests().await;
//        let app = Router::new()
//            .route("/envelopes/create", post(page))
//            .layer(user_for_tests(&ObjectId::new().to_hex()))
//            .with_state(shared_state);
//
//        let form_data = "name=test&amount=3400";
//        let request = Request::builder()
//            .method("POST")
//            .uri("/envelopes/create")
//            .header("content-type", "application/x-www-form-urlencoded")
//            .header("Accept", "text/vnd.turbo-stream.html")
//            .body(Body::from(form_data))
//            .unwrap();
//
//        let response = app.oneshot(request).await.unwrap();
//
//        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
//        assert_eq!(
//            response.headers().get("content-type").unwrap(),
//            "text/vnd.turbo-stream.html"
//        );
//    }
//}
