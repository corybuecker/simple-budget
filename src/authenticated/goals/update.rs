use super::{GoalForm, schema};
use crate::{
    SharedState,
    authenticated::UserExtension,
    errors::AppResponse,
    models::goal::{Goal, Recurrence},
    utilities::turbo,
};
use anyhow::anyhow;
use axum::{
    Extension, Form,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Redirect},
};
use chrono::{NaiveDateTime, NaiveTime};
use rust_decimal::{Decimal, prelude::FromPrimitive};
use std::str::FromStr;
use tera::Context;
use tokio_postgres::GenericClient;

pub async fn action(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
    Path(id): Path<i32>,
    headers: HeaderMap,
    Form(form): Form<GoalForm>,
) -> AppResponse {
    let json = serde_json::to_value(&form)?;
    let valid = jsonschema::validate(&schema(), &json);

    let is_turbo = turbo::is_turbo_request(&headers)?;
    
    match valid {
        Ok(_) => {}
        Err(validation_errors) => {
            let mut context = Context::new();

            context.insert("errors", &validation_errors.to_string());
            context.insert("id", &id);
            context.insert("name", &form.name);
            context.insert("target", &form.target);
            context.insert("target_date", &form.target_date);
            context.insert("recurrence", &form.recurrence);

            let template_name = turbo::get_template_name(is_turbo, "goals", "form");
            let content = shared_state.tera.render(&template_name, &context)?;

            return Ok(turbo::form_error_response(
                is_turbo,
                content,
                StatusCode::BAD_REQUEST,
            ));
        }
    }

    let mut goal = Goal::get_one(shared_state.pool.get().await?.client(), id, user.id).await?;

    goal.name = form.name.to_owned();
    goal.target = Decimal::from_f64(form.target.to_owned())
        .ok_or_else(|| anyhow!("could not parse decimal"))?;
    goal.recurrence = Recurrence::from_str(&form.recurrence).unwrap();
    goal.target_date = NaiveDateTime::new(form.target_date, NaiveTime::MIN).and_utc();
    goal.accumulated_amount = Decimal::ZERO;

    goal.update(shared_state.pool.get().await?.client()).await?;

    Ok(Redirect::to("/goals").into_response())
}

#[cfg(test)]
mod tests {
    use crate::{
        models::goal::{Goal, Recurrence},
        test_utils::state_for_tests,
    };
    use axum::http::{Method, Request, StatusCode};
    use chrono::Utc;
    use rust_decimal::Decimal;
    use tokio_postgres::GenericClient;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_update_goal() {
        let (shared_state, user_extension) = state_for_tests().await.unwrap();
        let user_id = user_extension.0.id;

        let goal = Goal {
            id: None,
            accumulated_amount: Decimal::new(100, 0),
            user_id,
            name: "Test Goal".to_string(),
            target: Decimal::new(1000, 0),
            target_date: Utc::now(),
            recurrence: Recurrence::Weekly,
        };

        let mut goal = goal
            .create(shared_state.pool.get().await.unwrap().client())
            .await
            .unwrap();

        goal.accumulated_amount = Decimal::new(100, 0);
        let goal = goal
            .update(shared_state.pool.get().await.unwrap().client())
            .await
            .unwrap();

        let request = Request::builder()
            .method(Method::POST)
            .uri(format!("/goals/{}", goal.id.unwrap()))
            .header("content-type", "application/x-www-form-urlencoded")
            .body(
                "name=Updated%20Goal&target=2000.0&target_date=2024-12-31&recurrence=weekly"
                    .to_string(),
            )
            .unwrap();

        // Create a test app and call the action
        let app = axum::Router::new()
            .route(
                "/goals/{id}",
                axum::routing::post(crate::authenticated::goals::update::action),
            )
            .with_state(shared_state.clone())
            .layer(user_extension);

        let response = app.oneshot(request).await.unwrap();

        // Assert the response
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers().get("location").unwrap(), "/goals");

        let goal = Goal::get_one(
            shared_state.pool.get().await.unwrap().client(),
            goal.id.unwrap(),
            user_id,
        )
        .await
        .unwrap();

        assert_eq!(goal.name, "Updated Goal");
        assert_eq!(goal.target, Decimal::new(2000, 0));
        assert_eq!(goal.accumulated_amount, Decimal::ZERO);
    }
}
