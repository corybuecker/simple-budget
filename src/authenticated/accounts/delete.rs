use crate::{
    SharedState, authenticated::UserExtension, errors::AppResponse, models::account::Account,
};
use axum::{
    Extension,
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse},
};
use tera::Context;
use tokio_postgres::GenericClient;

pub async fn modal(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
    Path(id): Path<i32>,
) -> AppResponse {
    let account = Account::get_one(shared_state.pool.get().await?.client(), id, user.id).await?;
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
) -> AppResponse {
    let account = Account::get_one(shared_state.pool.get().await?.client(), id, user.id).await?;
    account
        .delete(shared_state.pool.get().await?.client())
        .await?;

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
    use crate::test_utils::state_for_tests;
    use axum::Router;
    use axum::body::Body;
    use axum::http::Request;
    use rust_decimal::Decimal;
    use tokio_postgres::GenericClient;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_delete_modal() {
        let (shared_state, user_extension) = state_for_tests().await.unwrap();
        let account = Account {
            id: None,
            user_id: user_extension.0.id,
            name: "Test Account".to_string(),
            amount: Decimal::new(100, 0),
            debt: false,
        };

        let account = account
            .create(shared_state.pool.get().await.unwrap().client())
            .await
            .unwrap();

        let app = Router::new()
            .route("/accounts/{id}/delete", axum::routing::get(modal))
            .layer(user_extension)
            .with_state(shared_state.clone());

        let request = Request::builder()
            .uri(format!("/accounts/{}/delete", account.id.unwrap()))
            .method("GET")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_delete_action() {
        let (shared_state, user_extension) = state_for_tests().await.unwrap();
        let user_id = user_extension.0.id;
        let account = Account {
            id: None,
            user_id: user_extension.0.id,
            name: "Test Account".to_string(),
            amount: Decimal::new(100, 0),
            debt: false,
        };

        let account = account
            .create(shared_state.pool.get().await.unwrap().client())
            .await
            .unwrap();

        let app = Router::new()
            .route("/accounts/{id}", axum::routing::delete(action))
            .layer(user_extension)
            .with_state(shared_state.clone());

        let request = Request::builder()
            .uri(format!("/accounts/{}", account.id.unwrap()))
            .method("DELETE")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let deleted_account = Account::get_one(
            shared_state.pool.get().await.unwrap().client(),
            account.id.unwrap(),
            user_id,
        )
        .await;
        assert!(deleted_account.is_err());
    }
}
