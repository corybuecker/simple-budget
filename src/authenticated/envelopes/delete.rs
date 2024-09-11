use crate::{
    authenticated::UserExtension, errors::FormError, models::envelope::Envelope, SharedState,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    Extension,
};
use bson::doc;
use bson::oid::ObjectId;
use core::str::FromStr;

pub async fn action(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
    Path(id): Path<String>,
) -> Result<Response, FormError> {
    log::debug!("{:?}", user);
    let envelopes: mongodb::Collection<Envelope> = shared_state
        .mongo
        .default_database()
        .unwrap()
        .collection("envelopes");

    let filter = doc! {"_id": ObjectId::from_str(&id).unwrap(), "user_id": ObjectId::from_str(&user.id).unwrap()};

    let envelope = envelopes.find_one(filter.clone()).await?;

    let Some(_envelope) = envelope else {
        return Err(FormError {
            message: "could not find envelope".to_string(),
            status_code: Some(StatusCode::NOT_FOUND),
        });
    };

    let _ = envelopes.delete_one(filter).await;

    Ok(Redirect::to("/envelopes").into_response())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::envelope::Envelope;

    use crate::test_utils::{state_for_tests, user_for_tests};
    use axum::body::Body;
    use axum::http::Request;
    use axum::Router;

    use tower::ServiceExt;

    #[tokio::test]
    async fn test_delete_action() {
        let shared_state = state_for_tests().await;
        let envelopes = shared_state
            .mongo
            .default_database()
            .unwrap()
            .collection::<Envelope>("envelopes");

        envelopes
            .delete_many(doc! {"name": "delete_envelope"})
            .await
            .unwrap();

        let user_id = ObjectId::new();
        let envelope_id = ObjectId::new();

        let envelope = Envelope {
            _id: envelope_id.to_string(),
            user_id: user_id.to_string(),
            name: "delete_envelope".to_string(),
            amount: 100.0,
        };

        envelopes.insert_one(envelope).await.unwrap();

        // Create a router with the delete route
        let app = Router::new()
            .route("/envelopes/:id", axum::routing::delete(action))
            .layer(user_for_tests(&user_id.to_hex()))
            .with_state(shared_state);

        let request = Request::builder()
            .uri(format!("/envelopes/{}", envelope_id))
            .method("DELETE")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::SEE_OTHER);

        let deleted_envelope = envelopes.find_one(doc! {"_id": envelope_id}).await.unwrap();
        assert!(deleted_envelope.is_none());
    }
}
