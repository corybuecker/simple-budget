use crate::{
    authenticated::UserExtension, errors::FormError, models::account::Account, SharedState,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    Extension,
};
use mongodb::bson::{doc, oid::ObjectId};
use std::str::FromStr;
use tracing::info;

pub async fn action(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
    Path(id): Path<String>,
) -> Result<Response, FormError> {
    info!("{:#?}", user);

    let accounts: mongodb::Collection<Account> = shared_state
        .mongo
        .default_database()
        .unwrap()
        .collection("accounts");

    let filter = doc! {"_id": ObjectId::from_str(&id).unwrap(), "user_id": ObjectId::from_str(&user.id).unwrap()};
    let account = accounts.find_one(filter.clone()).await?;
    let Some(_) = account else {
        return Ok((StatusCode::NOT_FOUND, Html::from("Not Found")).into_response());
    };
    let _ = accounts.delete_one(filter).await?;

    Ok(Redirect::to("/accounts").into_response())
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::account::Account;
    use crate::mongo_client;
    use axum::body::Body;
    use axum::http::Request;
    use axum::Router;
    use axum_extra::extract::cookie::Key;
    use tera::Tera;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_delete_action() {
        let client = mongo_client().await.unwrap();
        let database = client.default_database().unwrap();
        let accounts = database.collection::<Account>("accounts");

        let user_id = ObjectId::new();
        let account_id = ObjectId::new();

        let account = Account {
            _id: account_id.to_string(),
            user_id: user_id.to_string(),
            name: "Test Account".to_string(),
            amount: 100.0,
            debt: false,
        };

        accounts.insert_one(account).await.unwrap();

        let tera = Tera::new("src/templates/**/*.html").expect("cannot initialize Tera");
        let shared_state = SharedState {
            mongo: client,
            key: Key::generate(),
            tera,
        };

        let user = UserExtension {
            id: user_id.to_string(),
            csrf: "test".to_string(),
        };

        // Create a router with the delete route
        let app = Router::new()
            .route("/accounts/:id", axum::routing::delete(action))
            .layer(Extension(user))
            .with_state(shared_state);

        let request = Request::builder()
            .uri(format!("/accounts/{}", account_id.to_string()))
            .method("DELETE")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::SEE_OTHER);

        let deleted_account = accounts.find_one(doc! {"_id": account_id}).await.unwrap();
        assert!(deleted_account.is_none());
    }
}
