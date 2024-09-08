use crate::{
    authenticated::UserExtension, errors::FormError, models::account::Account, SharedState,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Extension,
};
use mongodb::bson::{doc, oid::ObjectId};
use std::str::FromStr;
use tera::Context;
use tracing::info;

pub async fn modal(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
    Path(id): Path<String>,
) -> Result<Response, FormError> {
    let accounts: mongodb::Collection<Account> = shared_state
        .mongo
        .default_database()
        .unwrap()
        .collection("accounts");

    let filter = doc! {"_id": ObjectId::from_str(&id).unwrap(), "user_id": ObjectId::from_str(&user.id).unwrap()};

    let account = accounts.find_one(filter.clone()).await?;

    let Some(_) = account else {
        return Err(FormError {
            message: "could not find account".to_string(),
            status_code: Some(StatusCode::NOT_FOUND),
        });
    };
    let tera = shared_state.tera.clone();
    let mut context = Context::new();
    context.insert("account", &account);
    let content = tera.render("accounts/delete/confirm.html", &context)?;

    Ok(Html::from(content).into_response())
}

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

    let tera = shared_state.tera.clone();
    let mut context = Context::new();
    context.insert("account", &account);
    let content = tera.render("accounts/delete.html", &context)?;

    Ok((
        StatusCode::OK,
        [("content-type", "text/vnd.turbo-stream.html")],
        Html::from(content),
    )
        .into_response())
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::account::Account;

    use crate::test_utils::test_utils::{state_for_tests, user_for_tests};
    use axum::body::Body;
    use axum::http::Request;
    use axum::Router;

    use tower::ServiceExt;

    #[tokio::test]
    async fn test_delete_action() {
        let shared_state = state_for_tests().await;
        let database = shared_state.mongo.default_database().unwrap();
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

        let app = Router::new()
            .route("/accounts/:id", axum::routing::delete(action))
            .layer(user_for_tests(&user_id.to_hex()))
            .with_state(shared_state);

        let request = Request::builder()
            .uri(format!("/accounts/{}", account_id))
            .method("DELETE")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let deleted_account = accounts.find_one(doc! {"_id": account_id}).await.unwrap();
        assert!(deleted_account.is_none());
    }
}
