use super::{EditTemplate, GoalForm, schema};
use crate::{
    LayoutCtx, SharedState,
    authenticated::UserExtension,
    errors::AppResponse,
    models::goal::{Goal, Recurrence},
    utilities::responses::generate_response,
};
use anyhow::anyhow;
use askama::Template;
use axum::{
    Extension, Form,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use chrono::{NaiveDateTime, NaiveTime};
use rust_decimal::{Decimal, prelude::FromPrimitive};
use std::str::FromStr;
use uuid::Uuid;

pub async fn action(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
    Path(id): Path<Uuid>,
    Extension(layout): Extension<LayoutCtx>,
    Form(form): Form<GoalForm>,
) -> AppResponse {
    let client = shared_state.pool.get_client().await?;
    let json = serde_json::to_value(&form)?;

    if let Err(validation_errors) = jsonschema::validate(&schema(), &json) {
        let template = EditTemplate {
            layout,
            id,
            name: form.name,
            target: form.target.to_string(),
            target_date: form.target_date.to_string(),
            recurrence: form.recurrence,
            errors: validation_errors.to_string(),
        };
        return Ok(generate_response(
            template.render()?,
            StatusCode::BAD_REQUEST,
        ));
    }

    let mut goal = Goal::get_one(&client, id, user.id).await?;

    let new_recurrence = Recurrence::from_str(&form.recurrence).unwrap();

    goal.name = form.name.to_owned();
    goal.target = Decimal::from_f64(form.target.to_owned())
        .ok_or_else(|| anyhow!("could not parse decimal"))?;

    match new_recurrence {
        Recurrence::Never => match goal.recurrence {
            Recurrence::Never => {}
            _ => {
                goal.start_date = Some(chrono::Utc::now());
            }
        },
        _ => {
            goal.start_date = None;
        }
    }

    goal.recurrence = new_recurrence;
    goal.target_date = NaiveDateTime::new(form.target_date, NaiveTime::MIN).and_utc();
    goal.accumulated_amount = Decimal::ZERO;

    goal.update(&client).await?;

    Ok(Redirect::to("/goals").into_response())
}

#[cfg(test)]
mod tests {
    use rust_database_common::GenericClient;
    #[tokio::test]
    async fn test_update_goal_recurrence_monthly_to_never_sets_start_date() {
        use crate::models::goal::{Goal, Recurrence};
        use chrono::{TimeZone, Utc};
        use rust_decimal::Decimal;
        let (shared_state, user_extension, context_extension) =
            crate::test_utils::state_for_tests().await.unwrap();
        let client = shared_state.pool.get_client().await.unwrap();
        let user_id = user_extension.0.id;

        // Create a monthly goal
        let mut goal = Goal {
            id: None,
            name: "update_monthly_to_never".to_string(),
            target: Decimal::new(1000, 0),
            target_date: Utc.with_ymd_and_hms(2025, 12, 31, 0, 0, 0).unwrap(),
            user_id,
            accumulated_amount: Decimal::ZERO,
            recurrence: Recurrence::Monthly,
            start_date: None,
        };
        let created = goal.create(&client).await.unwrap();
        goal.id = created.id;

        let request = Request::builder()
            .method(Method::POST)
            .uri(format!("/goals/{}", goal.id.unwrap()))
            .header("content-type", "application/x-www-form-urlencoded")
            .body(
                "name=Updated%20Goal&target=2000.0&target_date=2024-12-31&recurrence=never"
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
            .layer(user_extension)
            .layer(context_extension);

        let response = app.oneshot(request).await.unwrap();

        // Assert the response
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers().get("location").unwrap(), "/goals");

        let goal = client
            .query_one(
                "SELECT * FROM goals WHERE user_id = $1 LIMIT 1",
                &[&user_id],
            )
            .await
            .unwrap();

        assert_eq!(goal.get::<_, String>("name"), "Updated Goal");
        let start_date_str = goal
            .get::<_, chrono::DateTime<chrono::Utc>>("start_date")
            .to_rfc3339();
        assert!(chrono::DateTime::parse_from_rfc3339(&start_date_str).is_ok());
    }
    use crate::{
        models::goal::{Goal, Recurrence},
        test_utils::state_for_tests,
    };
    use axum::http::{Method, Request, StatusCode};
    use chrono::Utc;
    use rust_decimal::Decimal;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_update_goal() {
        let (shared_state, user_extension, context_extension) = state_for_tests().await.unwrap();
        let user_id = user_extension.0.id;

        let goal = Goal {
            id: None,
            accumulated_amount: Decimal::new(100, 0),
            user_id,
            name: "Test Goal".to_string(),
            target: Decimal::new(1000, 0),
            target_date: Utc::now(),
            recurrence: Recurrence::Weekly,
            start_date: None,
        };
        let client = shared_state.pool.get_client().await.unwrap();

        let mut goal = goal.create(&client).await.unwrap();

        goal.accumulated_amount = Decimal::new(100, 0);
        let goal = goal.update(&client).await.unwrap();

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
            .layer(user_extension)
            .layer(context_extension);

        let response = app.oneshot(request).await.unwrap();
        let client = shared_state.pool.get_client().await.unwrap();

        // Assert the response
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers().get("location").unwrap(), "/goals");

        let goal = Goal::get_one(&client, goal.id.unwrap(), user_id)
            .await
            .unwrap();

        assert_eq!(goal.name, "Updated Goal");
        assert_eq!(goal.target, Decimal::new(2000, 0));
        assert_eq!(goal.accumulated_amount, Decimal::ZERO);
    }
}
