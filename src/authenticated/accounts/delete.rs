use crate::{
    HandlebarsContext, SharedState,
    authenticated::UserExtension,
    errors::AppResponse,
    models::account::Account,
    utilities::responses::{ResponseFormat, generate_response, get_response_format},
};
use axum::{
    Extension, Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use handlebars::to_json;

pub async fn modal(
    shared_state: State<SharedState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
    Extension(user): Extension<UserExtension>,
    Extension(context): Extension<HandlebarsContext>,
) -> AppResponse {
    let client = shared_state.pool.get_client().await?;
    let account = Account::get_one(&client, id, user.id).await?;
    let response_format = get_response_format(&headers)?;

    match response_format {
        ResponseFormat::Html => {
            let mut context = context.clone();
            context.insert(
                "prompt".to_string(),
                to_json("Are you sure you want to delete this account?"),
            );
            context.insert("action".to_string(), to_json(format!("/accounts/{}", id)));
            context.insert("entity".to_string(), to_json(account.name));
            context.insert("partial".to_string(), to_json("delete_confirmation"));
            Ok(generate_response(
                &response_format,
                shared_state
                    .handlebars
                    .render("delete_confirmation", &context)?,
                StatusCode::OK,
            ))
        }
        ResponseFormat::Turbo => Ok(StatusCode::NOT_ACCEPTABLE.into_response()),
        ResponseFormat::Json => Ok(generate_response(
            &response_format,
            Json(account),
            StatusCode::OK,
        )),
    }
}

pub async fn action(
    shared_state: State<SharedState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
    Extension(user): Extension<UserExtension>,
    Extension(context): Extension<HandlebarsContext>,
) -> AppResponse {
    let client = shared_state.pool.get_client().await?;
    let account = Account::get_one(&client, id, user.id).await?;

    account.delete(&client).await?;

    let response_format = get_response_format(&headers)?;
    match response_format {
        ResponseFormat::Html => Ok(StatusCode::NOT_ACCEPTABLE.into_response()),
        ResponseFormat::Json => Ok(generate_response(
            &response_format,
            Json(account),
            StatusCode::OK,
        )),
        ResponseFormat::Turbo => {
            let mut context = context.clone();
            context.insert("account".to_string(), to_json(&account));

            Ok(generate_response(
                &response_format,
                shared_state
                    .handlebars
                    .render("accounts/delete", &context)?,
                StatusCode::OK,
            ))
        }
    }
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
            .header("Accept", "turbo")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let deleted_account = Account::get_one(&client, account.id.unwrap(), user_id).await;
        assert!(deleted_account.is_err());
    }
}
