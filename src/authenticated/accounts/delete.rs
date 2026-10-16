use crate::{
    LayoutCtx, SharedState, authenticated::UserExtension, errors::AppResponse,
    models::account::Account, templates::DeleteConfirmation,
    utilities::responses::generate_response,
};
use askama::Template;
use axum::{
    Extension,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use uuid::Uuid;

pub async fn modal(
    shared_state: State<SharedState>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<UserExtension>,
    Extension(layout): Extension<LayoutCtx>,
) -> AppResponse {
    let client = shared_state.pool.get_client().await?;
    let account = Account::get_one(&client, id, user.id).await?;

    let template = DeleteConfirmation {
        layout,
        prompt: "Are you sure you want to delete this account?".to_string(),
        action: format!("/accounts/{}/delete", id),
        cancel: "/accounts".to_string(),
        entity: account.name,
    };

    Ok(generate_response(template.render()?, StatusCode::OK))
}

pub async fn action(
    shared_state: State<SharedState>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<UserExtension>,
) -> AppResponse {
    let client = shared_state.pool.get_client().await?;
    let account = Account::get_one(&client, id, user.id).await?;

    account.delete(&client).await?;

    Ok(Redirect::to("/accounts").into_response())
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
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_delete_modal() {
        let (shared_state, user_extension, context_extension) = state_for_tests().await.unwrap();
        let account = Account {
            id: None,
            user_id: user_extension.0.id,
            name: "Test Account".to_string(),
            amount: Decimal::new(100, 0),
            debt: false,
        };
        let client = shared_state.pool.get_client().await.unwrap();
        let account = account.create(&client).await.unwrap();

        let app = Router::new()
            .route("/accounts/{id}/delete", axum::routing::get(modal))
            .layer(user_extension)
            .layer(context_extension)
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
        let (shared_state, user_extension, context_extension) = state_for_tests().await.unwrap();
        let client = shared_state.pool.get_client().await.unwrap();
        let user_id = user_extension.0.id;
        let account = Account {
            id: None,
            user_id: user_extension.0.id,
            name: "Test Account".to_string(),
            amount: Decimal::new(100, 0),
            debt: false,
        };

        let account = account.create(&client).await.unwrap();

        let app = Router::new()
            .route("/accounts/{id}", axum::routing::delete(action))
            .layer(user_extension)
            .layer(context_extension)
            .with_state(shared_state.clone());

        let request = Request::builder()
            .uri(format!("/accounts/{}", account.id.unwrap()))
            .method("DELETE")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers().get("location").unwrap(), "/accounts");

        let deleted_account = Account::get_one(&client, account.id.unwrap(), user_id).await;
        assert!(deleted_account.is_err());
    }
}
