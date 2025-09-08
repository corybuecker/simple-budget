use super::{AccountForm, schema};
use crate::{
    HandlebarsContext, SharedState,
    authenticated::UserExtension,
    errors::AppResponse,
    models::account::Account,
    utilities::responses::{self, generate_response, get_response_format},
};
use anyhow::anyhow;
use axum::{
    Extension, Form, Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Redirect},
};
use handlebars::to_json;
use rust_decimal::{Decimal, prelude::FromPrimitive};
use tokio_postgres::GenericClient;

pub async fn action(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
    Path(id): Path<i32>,
    headers: HeaderMap,
    Extension(context): Extension<HandlebarsContext>,
    Form(form): Form<AccountForm>,
) -> AppResponse {
    let json = serde_json::to_value(&form)?;
    let valid = jsonschema::validate(&schema(), &json);
    let response_format = responses::get_response_format(&headers)?;

    match valid {
        Ok(_) => {}
        Err(validation_errors) => {
            let mut context = context.clone();

            context.insert("errors".to_string(), to_json(validation_errors.to_string()));
            context.insert("id".to_string(), to_json(id));
            context.insert("name".to_string(), to_json(&form.name));
            context.insert("amount".to_string(), to_json(form.amount));
            context.insert("debt".to_string(), to_json(form.debt));

            match response_format {
                responses::ResponseFormat::Html => {
                    context.insert("partial".to_string(), to_json("accounts/form"));
                    return Ok(responses::generate_response(
                        &responses::ResponseFormat::Html,
                        shared_state.handlebars.render("layout", &context)?,
                        StatusCode::BAD_REQUEST,
                    ));
                }
                responses::ResponseFormat::Turbo => {
                    return Ok(responses::generate_response(
                        &response_format,
                        shared_state.handlebars.render("accounts/form", &context)?,
                        StatusCode::BAD_REQUEST,
                    ));
                }
                responses::ResponseFormat::Json => {
                    return Ok(responses::generate_response(
                        &response_format,
                        serde_json::to_string(&context)?,
                        StatusCode::BAD_REQUEST,
                    ));
                }
            }
        }
    }

    let mut account =
        Account::get_one(shared_state.pool.get().await?.client(), id, user.id).await?;
    account.name = form.name.clone();
    account.amount =
        Decimal::from_f64(form.amount).ok_or_else(|| anyhow!("could not parse decimal"))?;
    account.debt = form.debt.unwrap_or(false);
    account
        .update(shared_state.pool.get().await?.client())
        .await?;

    match get_response_format(&headers)? {
        responses::ResponseFormat::Html | responses::ResponseFormat::Turbo => {
            Ok(Redirect::to("/accounts").into_response())
        }
        responses::ResponseFormat::Json => Ok(generate_response(
            &responses::ResponseFormat::Json,
            Json(account),
            StatusCode::OK,
        )),
    }
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
        let (shared_state, user_extension, context_extension) = state_for_tests().await.unwrap();
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
            .layer(user_extension)
            .layer(context_extension);

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
