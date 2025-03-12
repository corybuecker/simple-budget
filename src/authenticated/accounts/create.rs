use super::AccountForm;
use crate::errors::FormError;
use crate::{SharedState, authenticated::UserExtension, models::account::Account};
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
    Form(form): Form<AccountForm>,
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
            context.insert("name", &form.name);
            context.insert("amount", &form.amount);
            context.insert("debt", &form.debt);

            let content = shared_state.tera.render(
                if turbo {
                    "accounts/form.turbo.html"
                } else {
                    "accounts/new.html"
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

    let mut account_record = Account {
        id: None,
        name: form.name.to_owned(),
        amount: form.amount.to_owned(),
        debt: form.debt.unwrap_or(false),
        user_id: Some(user.id),
    };

    account_record.create(&shared_state.client).await?;

    Ok(Redirect::to("/accounts").into_response())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::state_for_tests;
    use axum::Router;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use axum::routing::post;
    use std::str::from_utf8;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_create_account_success() {
        let (shared_state, user_extension) = state_for_tests().await.unwrap();
        let user_id = user_extension.0.id;
        let client = &shared_state.client.clone();

        let app = Router::new()
            .route("/accounts/create", post(page))
            .with_state(shared_state)
            .layer(user_extension);

        let form_data = "name=test_create_account_success&amount=100.00&debt=false";
        let request = Request::builder()
            .method("POST")
            .uri("/accounts/create")
            .header("content-type", "application/x-www-form-urlencoded")
            .body(Body::from(form_data))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers().get("location").unwrap(), "/accounts");

        let account = client
            .query_one(
                "SELECT * FROM accounts WHERE user_id = $1 LIMIT 1",
                &[&user_id],
            )
            .await
            .unwrap();

        assert_eq!(
            account.get::<_, String>("name"),
            "test_create_account_success"
        )
    }

    #[tokio::test]
    async fn test_create_account_validation_error() {
        let (shared_state, user_extension) = state_for_tests().await.unwrap();

        let app = Router::new()
            .route("/accounts/create", post(page))
            .with_state(shared_state)
            .layer(user_extension);

        let form_data = "name=test&amount=1&debt=false";
        let request = Request::builder()
            .method("POST")
            .uri("/accounts/create")
            .header("content-type", "application/x-www-form-urlencoded")
            .body(Body::from(form_data))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        let (parts, body) = response.into_parts();
        let bytes = to_bytes(body, usize::MAX).await.unwrap();
        let body_str = from_utf8(&bytes).unwrap().to_string();

        assert_eq!(parts.status, StatusCode::BAD_REQUEST);
        assert!(body_str.contains("test"))
    }

    #[tokio::test]
    async fn test_create_account_turbo_stream() {
        let (shared_state, user_extension) = state_for_tests().await.unwrap();

        let app = Router::new()
            .route("/accounts/create", post(page))
            .with_state(shared_state)
            .layer(user_extension);

        let form_data = "name=test&amount=1&debt=false";
        let request = Request::builder()
            .method("POST")
            .uri("/accounts/create")
            .header("content-type", "application/x-www-form-urlencoded")
            .header("Accept", "text/vnd.turbo-stream.html")
            .body(Body::from(form_data))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "text/vnd.turbo-stream.html"
        );
    }
}
