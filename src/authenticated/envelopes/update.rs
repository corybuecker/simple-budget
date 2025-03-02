use super::EnvelopeForm;
use crate::{
    SharedState, authenticated::UserExtension, errors::FormError, models::envelope::Envelope,
};
use anyhow::Result;
use axum::{
    Extension, Form,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
};
use tera::Context;
use validator::Validate;

pub async fn action(
    shared_state: State<SharedState>,
    Extension(user): Extension<UserExtension>,
    Path(id): Path<i32>,
    headers: HeaderMap,
    form: Form<EnvelopeForm>,
) -> Result<Response, FormError> {
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

    let envelope = Envelope {
        id: Some(id),
        name: form.name.to_owned(),
        amount: form.amount.to_owned(),
        user_id: Some(user.id),
    };

    envelope.update(&shared_state.client).await?;

    Ok(Redirect::to("/envelopes").into_response())
}

//#[cfg(test)]
//mod tests {
//    use crate::{
//        models::envelope::Envelope,
//        test_utils::{state_for_tests, user_for_tests},
//    };
//    use axum::http::{Method, Request, StatusCode};
//
//    use mongodb::bson::doc;
//    use tower::ServiceExt;
//
//    #[tokio::test]
//    async fn test_update_envelope() {
//        let shared_state = state_for_tests().await;
//        // Set up the database connection
//        let db = shared_state.mongo.default_database().unwrap();
//        let envelopes_collection: mongodb::Collection<Envelope> = db.collection("envelopes");
//
//        // Create a test envelope
//        let user_id = mongodb::bson::oid::ObjectId::new();
//        let envelope_id = mongodb::bson::oid::ObjectId::new();
//        let test_envelope = Envelope {
//            _id: envelope_id.to_string(),
//            user_id: user_id.to_string(),
//            name: "Test Envelope".to_string(),
//            amount: 100.0,
//        };
//        envelopes_collection
//            .insert_one(test_envelope)
//            .await
//            .unwrap();
//
//        let request = Request::builder()
//            .method(Method::POST)
//            .uri(format!("/envelopes/{}", envelope_id.to_hex()))
//            .header("content-type", "application/x-www-form-urlencoded")
//            .body("name=Updated%20Envelope&amount=200.0".to_string())
//            .unwrap();
//
//        // Create a test app and call the action
//        let app = axum::Router::new()
//            .route(
//                "/envelopes/{id}",
//                axum::routing::post(crate::authenticated::envelopes::update::action),
//            )
//            .with_state(shared_state)
//            .layer(user_for_tests(&user_id.to_hex()));
//
//        let response = app.oneshot(request).await.unwrap();
//
//        // Assert the response
//        assert_eq!(response.status(), StatusCode::SEE_OTHER);
//        assert_eq!(response.headers().get("location").unwrap(), "/envelopes");
//
//        // Verify the envelope was updated in the database
//        let updated_envelope = envelopes_collection
//            .find_one(doc! {"_id": envelope_id})
//            .await
//            .unwrap()
//            .unwrap();
//
//        assert_eq!(updated_envelope.name, "Updated Envelope");
//        assert_eq!(updated_envelope.amount, 200.0);
//
//        // Clean up
//        envelopes_collection
//            .delete_one(doc! {"_id": envelope_id})
//            .await
//            .unwrap();
//    }
//}
