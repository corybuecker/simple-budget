use super::AccountForm;
use crate::{
    authenticated::UserExtension, errors::FormError, models::account::Account, SharedState,
};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    Extension, Form,
};
use mongodb::bson::{doc, oid::ObjectId};
use std::str::FromStr;
use tera::Context;
use validator::Validate;

pub async fn action(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
    Path(id): Path<String>,
    headers: HeaderMap,
    form: Form<AccountForm>,
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

    let accounts: mongodb::Collection<Account> = shared_state
        .mongo
        .default_database()
        .unwrap()
        .collection("accounts");

    let filter = doc! {"_id": ObjectId::from_str(&id).unwrap(), "user_id": ObjectId::from_str(&user.id).unwrap()};

    let account = accounts.find_one(filter.clone()).await?;

    let Some(mut account) = account else {
        return Err(FormError {
            message: "could not find account".to_string(),
            status_code: Some(StatusCode::NOT_FOUND),
        });
    };

    account.name = form.name.clone();
    account.amount = form.amount;
    account.debt = form.debt.unwrap_or(false);

    let _ = accounts.replace_one(filter, account).await?;

    Ok(Redirect::to("/accounts").into_response())
}

#[cfg(test)]
mod tests {
    use crate::{
        authenticated::UserExtension, models::account::Account, mongo_client, SharedState,
    };
    use axum::{
        http::{Method, Request, StatusCode},
        Extension,
    };
    use axum_extra::extract::cookie::Key;
    use mongodb::bson::doc;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_update_account() {
        // Set up the database connection
        let client = mongo_client().await.unwrap();
        let db = client.default_database().unwrap();
        let accounts_collection: mongodb::Collection<Account> = db.collection("accounts");

        // Create a test account
        let user_id = mongodb::bson::oid::ObjectId::new();
        let account_id = mongodb::bson::oid::ObjectId::new();
        let test_account = Account {
            _id: account_id.to_string(),
            user_id: user_id.to_string(),
            name: "Test Account".to_string(),
            amount: 100.0,
            debt: false,
        };
        accounts_collection.insert_one(test_account).await.unwrap();

        // Set up the SharedState
        let shared_state = SharedState {
            mongo: client,
            key: Key::generate(),
            tera: tera::Tera::new("templates/**/*").unwrap(),
        };

        let request = Request::builder()
            .method(Method::POST)
            .uri(format!("/accounts/{}", account_id.to_hex()))
            .header("content-type", "application/x-www-form-urlencoded")
            .body("name=Updated%20Account&debt=true&amount=200.0".to_string())
            .unwrap();

        // Create a test app and call the action
        let app = axum::Router::new()
            .route(
                "/accounts/:id",
                axum::routing::post(crate::authenticated::accounts::update::action),
            )
            .with_state(shared_state)
            .layer(Extension(UserExtension {
                id: user_id.to_string(),
                csrf: "test".to_string(),
            }));

        let response = app.oneshot(request).await.unwrap();

        // Assert the response
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers().get("location").unwrap(), "/accounts");

        // Verify the account was updated in the database
        let updated_account = accounts_collection
            .find_one(doc! {"_id": account_id})
            .await
            .unwrap()
            .unwrap();

        assert_eq!(updated_account.name, "Updated Account");
        assert_eq!(updated_account.amount, 200.0);
        assert!(updated_account.debt);

        // Clean up
        accounts_collection
            .delete_one(doc! {"_id": account_id})
            .await
            .unwrap();
    }
}
