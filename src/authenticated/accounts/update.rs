use super::{AccountForm, EditTemplate, schema};
use crate::{
    LayoutCtx, SharedState, authenticated::UserExtension, errors::AppResponse,
    models::account::Account, utilities::responses::generate_response,
};
use anyhow::anyhow;
use askama::Template;
use axum::{
    Extension, Form,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use rust_decimal::{Decimal, prelude::FromPrimitive};
use uuid::Uuid;

pub async fn action(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
    Path(id): Path<Uuid>,
    Extension(layout): Extension<LayoutCtx>,
    Form(form): Form<AccountForm>,
) -> AppResponse {
    let client = shared_state.pool.get_client().await?;
    let json = serde_json::to_value(&form)?;

    if let Err(validation_errors) = jsonschema::validate(&schema(), &json) {
        let template = EditTemplate {
            layout,
            id,
            name: form.name,
            amount: form.amount.to_string(),
            debt: form.debt.unwrap_or(false),
            errors: validation_errors.to_string(),
        };
        return Ok(generate_response(
            template.render()?,
            StatusCode::BAD_REQUEST,
        ));
    }

    let mut account = Account::get_one(&client, id, user.id).await?;
    account.name = form.name.clone();
    account.amount =
        Decimal::from_f64(form.amount).ok_or_else(|| anyhow!("could not parse decimal"))?;
    account.debt = form.debt.unwrap_or(false);
    account.update(&client).await?;

    Ok(Redirect::to("/accounts").into_response())
}

#[cfg(test)]
mod tests {
    use crate::{models::account::Account, test_utils::state_for_tests};
    use axum::http::{Method, Request, StatusCode};
    use rust_decimal::Decimal;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_update_account() {
        let (shared_state, user_extension, context_extension) = state_for_tests().await.unwrap();
        let user_id = user_extension.0.id;

        let account = Account {
            id: None,
            user_id,
            name: "Test Account".to_string(),
            amount: Decimal::new(100, 0),
            debt: false,
        };

        let client = shared_state.pool.get_client().await.unwrap();

        let account = account.create(&client).await.unwrap();

        let request = Request::builder()
            .method(Method::POST)
            .uri(format!("/accounts/{}", account.id.unwrap()))
            .header("content-type", "application/x-www-form-urlencoded")
            .body("name=Updated%20Account&debt=true&amount=200.0".to_string())
            .unwrap();

        // Create a test app and call the action
        let app = axum::Router::new()
            .route(
                "/accounts/{id}",
                axum::routing::post(crate::authenticated::accounts::update::action),
            )
            .with_state(shared_state.clone())
            .layer(user_extension)
            .layer(context_extension);

        let response = app.oneshot(request).await.unwrap();

        // Assert the response
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers().get("location").unwrap(), "/accounts");

        let account = Account::get_one(&client, account.id.unwrap(), user_id)
            .await
            .unwrap();

        assert_eq!(account.name, "Updated Account");
        assert_eq!(account.amount, Decimal::new(200, 0));
        assert!(account.debt);
    }
}
