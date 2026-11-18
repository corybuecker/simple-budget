use super::AccountForm;
use crate::LayoutCtx;
use crate::authenticated::accounts::schema;
use crate::errors::AppResponse;
use crate::{
    SharedState, authenticated::UserExtension, models::account::Account, utilities::responses,
};
use anyhow::anyhow;
use askama::Template;
use axum::{
    Extension, Form,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;

#[derive(Template)]
#[template(path = "accounts/new.html")]
struct New {
    #[allow(unused)]
    layout: LayoutCtx,

    name: String,
    amount: String,
    debt: bool,

    errors: String,
}
pub async fn action(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
    Extension(layout): Extension<LayoutCtx>,
    Form(form): Form<AccountForm>,
) -> AppResponse {
    tracing::debug!("testing");
    let json = serde_json::to_value(&form)?;
    let valid = jsonschema::validate(&schema(), &json);

    match valid {
        Ok(_) => {}
        Err(validation_errors) => {
            let new = New {
                layout,
                name: form.name,
                amount: form.amount.to_string(),
                debt: form.debt.unwrap_or(false),
                errors: validation_errors.to_string(),
            };
            return Ok(responses::generate_response(
                &responses::ResponseFormat::Html,
                new.render()?,
                StatusCode::BAD_REQUEST,
            ));
        }
    }

    let account = Account {
        id: None,
        name: form.name.to_owned(),
        amount: Decimal::from_f64(form.amount.to_owned())
            .ok_or_else(|| anyhow!("could not parse decimal"))?,
        debt: form.debt.unwrap_or(false),
        user_id: user.id,
    };
    let client = &shared_state.pool.get_client().await?;
    account.create(client).await?;

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
    use rust_database_common::GenericClient;
    use std::str::from_utf8;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_create_account_success() {
        let (shared_state, user_extension, context_extension) = state_for_tests().await.unwrap();
        let user_id = user_extension.0.id;

        let app = Router::new()
            .route("/accounts/create", post(action))
            .with_state(shared_state.clone())
            .layer(user_extension)
            .layer(context_extension);
        let pool = shared_state.pool;
        let client = pool.get_client().await.unwrap();
        let form_data = "name=test_create_account_success&amount=100.00&debt=true";
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
        let (shared_state, user_extension, context_extension) = state_for_tests().await.unwrap();

        let app = Router::new()
            .route("/accounts/create", post(action))
            .with_state(shared_state)
            .layer(user_extension)
            .layer(context_extension);

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
        println!("body_str: {}", body_str);
        assert!(body_str.contains("test"))
    }

    #[tokio::test]
    async fn test_create_account_turbo_stream() {
        let (shared_state, user_extension, context_extension) = state_for_tests().await.unwrap();

        let app = Router::new()
            .route("/accounts/create", post(action))
            .with_state(shared_state)
            .layer(user_extension)
            .layer(context_extension);

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
