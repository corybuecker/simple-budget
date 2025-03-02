use crate::{
    SharedState, authenticated::UserExtension, errors::FormError, models::account::Account,
};
use axum::{
    Extension,
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use tera::Context;

pub async fn modal(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
    Path(id): Path<i32>,
) -> Result<Response, FormError> {
    let account = Account::get_one(&shared_state.client, id, user.id).await?;
    let tera = shared_state.tera.clone();
    let mut context = Context::new();
    context.insert("account", &account);
    let content = tera.render("accounts/delete/confirm.html", &context)?;

    Ok(Html::from(content).into_response())
}

pub async fn action(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
    Path(id): Path<i32>,
) -> Result<Response, FormError> {
    let account = Account::get_one(&shared_state.client, id, user.id).await?;
    account.delete(&shared_state.client).await?;

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

//#[cfg(test)]
//mod tests {
//    use super::*;
//    use crate::models::account::Account;
//
//    use crate::test_utils::{state_for_tests, user_for_tests};
//    use axum::Router;
//    use axum::body::Body;
//    use axum::http::Request;
//
//    use tower::ServiceExt;
//
//    #[tokio::test]
//    async fn test_delete_action() {
//        let shared_state = state_for_tests().await;
//        let database = shared_state.mongo.default_database().unwrap();
//        let accounts = database.collection::<Account>("accounts");
//
//        let user_id = ObjectId::new();
//        let account_id = ObjectId::new();
//
//        let account = Account {
//            _id: account_id.to_string(),
//            user_id: user_id.to_string(),
//            name: "Test Account".to_string(),
//            amount: 100.0,
//            debt: false,
//        };
//
//        accounts.insert_one(account).await.unwrap();
//
//        let app = Router::new()
//            .route("/accounts/{id}", axum::routing::delete(action))
//            .layer(user_for_tests(&user_id.to_hex()))
//            .with_state(shared_state);
//
//        let request = Request::builder()
//            .uri(format!("/accounts/{}", account_id))
//            .method("DELETE")
//            .body(Body::empty())
//            .unwrap();
//
//        let response = app.oneshot(request).await.unwrap();
//
//        assert_eq!(response.status(), StatusCode::OK);
//
//        let deleted_account = accounts.find_one(doc! {"_id": account_id}).await.unwrap();
//        assert!(deleted_account.is_none());
//    }
//}
