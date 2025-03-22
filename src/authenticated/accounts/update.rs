use super::{AccountForm, schema};
use crate::{
    SharedState, authenticated::UserExtension, errors::AppResponse, models::account::Account,
};
use axum::{
    Extension, Form,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect},
};
use rust_decimal::{Decimal, prelude::FromPrimitive};
use tera::Context;
use tokio_postgres::GenericClient;

pub async fn action(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
    Path(id): Path<i32>,
    headers: HeaderMap,
    Form(form): Form<AccountForm>,
) -> AppResponse {
    let json = serde_json::to_value(&form)?;
    let valid = jsonschema::validate(&schema(), &json);
    let mut turbo = false;
    let accept = headers.get("Accept");
    if let Some(accept) = accept {
        if accept.to_str().unwrap().contains("turbo") {
            turbo = true;
        }
    }
    match valid {
        Ok(_) => {}
        Err(validation_errors) => {
            let mut context = Context::new();

            context.insert("errors", &validation_errors.to_string());
            context.insert("id", &id);
            context.insert("name", &form.name);
            context.insert("amount", &form.amount);
            context.insert("debt", &form.debt);
            let content = shared_state.tera.render(
                if turbo {
                    "accounts/form.turbo.html"
                } else {
                    "accounts/edit.html"
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

    let mut account =
        Account::get_one(shared_state.pool.get().await?.client(), id, user.id).await?;
    account.name = form.name.clone();
    account.amount = Decimal::from_f64(form.amount).expect("valid decimal");
    account.debt = form.debt.unwrap_or(false);
    account
        .update(shared_state.pool.get().await?.client())
        .await?;

    Ok(Redirect::to("/accounts").into_response())
}

#[cfg(test)]
mod tests {
    use crate::{models::account::Account, test_utils::state_for_tests};
    use axum::http::{Method, Request, StatusCode};
    use rust_decimal::Decimal;
    use tokio_postgres::GenericClient;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_update_account() {
        let (shared_state, user_extension) = state_for_tests().await.unwrap();
        let user_id = user_extension.0.id;

        let account = Account {
            id: None,
            user_id,
            name: "Test Account".to_string(),
            amount: Decimal::new(100, 0),
            debt: false,
        };

        let account = account
            .create(shared_state.pool.get().await.unwrap().client())
            .await
            .unwrap();

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
            .layer(user_extension);

        let response = app.oneshot(request).await.unwrap();

        // Assert the response
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers().get("location").unwrap(), "/accounts");

        let account = Account::get_one(
            shared_state.pool.get().await.unwrap().client(),
            account.id.unwrap(),
            user_id,
        )
        .await
        .unwrap();

        assert_eq!(account.name, "Updated Account");
        assert_eq!(account.amount, Decimal::new(200, 0));
        assert!(account.debt);
    }
}
