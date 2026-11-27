use crate::{
    SharedState,
    authenticated::UserExtension,
    errors::AppResponse,
    models::goal::{Goal, Recurrence},
};
use axum::{
    Extension,
    extract::{Path, State},
    response::{IntoResponse, Redirect},
};
use rust_decimal::Decimal;

pub async fn action(
    shared_state: State<SharedState>,
    Path(_recurrence): Path<String>,
    Extension(user): Extension<UserExtension>,
) -> AppResponse {
    let client = shared_state.pool.get_client().await?;

    for mut goal in Goal::get_all(&client, user.id).await? {
        if goal.recurrence.eq(&Recurrence::Monthly) {
            goal.accumulated_amount = Decimal::ZERO;
            goal.update(&client).await?;
        }
    }

    Ok(Redirect::to("/goals").into_response())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::goal::{Goal, Recurrence};
    use crate::models::user::preferences::Preferences;
    use crate::test_utils::state_for_tests;
    use axum::Router;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use axum::routing::post;
    use chrono::{Duration, Utc};
    use postgres_types::Json;
    use rust_database_common::GenericClient;
    use rust_decimal::Decimal;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_resets_action_html_redirect() {
        let (shared_state, user_extension, context_extension) = state_for_tests().await.unwrap();
        let client = shared_state.pool.get_client().await.unwrap();
        let user_id = user_extension.0.id;

        // Create a monthly goal that will be reset
        let goal = Goal {
            id: None,
            user_id,
            name: "Test Monthly Goal".to_string(),
            recurrence: Recurrence::Monthly,
            target: Decimal::new(1000, 0),
            target_date: Utc::now() + Duration::days(30),
            accumulated_amount: Decimal::new(500, 0), // This should be reset to 0
            start_date: None,
        };
        let mut goal = goal.create(&client).await.unwrap();
        // Set accumulated amount after creation since create() sets it to ZERO
        goal.accumulated_amount = Decimal::new(500, 0);
        let goal = goal.update(&client).await.unwrap();

        let app = Router::new()
            .route("/goals/resets/{recurrence}", post(action))
            .layer(user_extension)
            .layer(context_extension)
            .with_state(shared_state.clone());

        let request = Request::builder()
            .method("POST")
            .uri("/goals/resets/monthly")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers().get("location").unwrap(), "/goals");

        // Verify the goal was reset
        let updated_goal = Goal::get_one(&client, goal.id.unwrap(), user_id)
            .await
            .unwrap();
        assert_eq!(updated_goal.accumulated_amount, Decimal::ZERO);
    }

    #[tokio::test]
    async fn test_resets_action_only_resets_monthly_goals() {
        let (shared_state, user_extension, context_extension) = state_for_tests().await.unwrap();
        let client = shared_state.pool.get_client().await.unwrap();
        let user_id = user_extension.0.id;

        // Create goals with different recurrences
        let monthly_goal = Goal {
            id: None,
            user_id,
            name: "Monthly Goal".to_string(),
            recurrence: Recurrence::Monthly,
            target: Decimal::new(1000, 0),
            target_date: Utc::now() + Duration::days(30),
            accumulated_amount: Decimal::new(750, 0),
            start_date: None,
        };
        let mut monthly_goal = monthly_goal.create(&client).await.unwrap();
        // Set accumulated amount after creation since create() sets it to ZERO
        monthly_goal.accumulated_amount = Decimal::new(750, 0);
        let monthly_goal = monthly_goal.update(&client).await.unwrap();

        let weekly_goal = Goal {
            id: None,
            user_id,
            name: "Weekly Goal".to_string(),
            recurrence: Recurrence::Weekly,
            target: Decimal::new(200, 0),
            target_date: Utc::now() + Duration::days(7),
            accumulated_amount: Decimal::new(100, 0),
            start_date: None,
        };
        let mut weekly_goal = weekly_goal.create(&client).await.unwrap();
        // Set accumulated amount after creation since create() sets it to ZERO
        weekly_goal.accumulated_amount = Decimal::new(100, 0);
        let weekly_goal = weekly_goal.update(&client).await.unwrap();

        let app = Router::new()
            .route("/goals/resets/{recurrence}", post(action))
            .layer(user_extension)
            .layer(context_extension)
            .with_state(shared_state.clone());

        let request = Request::builder()
            .method("POST")
            .uri("/goals/resets/monthly")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::SEE_OTHER);

        // Verify only monthly goals were reset
        let updated_monthly = Goal::get_one(&client, monthly_goal.id.unwrap(), user_id)
            .await
            .unwrap();
        assert_eq!(updated_monthly.accumulated_amount, Decimal::ZERO);

        let updated_weekly = Goal::get_one(&client, weekly_goal.id.unwrap(), user_id)
            .await
            .unwrap();
        assert_eq!(updated_weekly.accumulated_amount, Decimal::new(100, 0)); // Should not be reset
    }

    #[tokio::test]
    async fn test_resets_action_with_user_preferences() {
        let (shared_state, user_extension, context_extension) = state_for_tests().await.unwrap();
        let client = shared_state.pool.get_client().await.unwrap();
        let user_id = user_extension.0.id;

        // Update user preferences
        let preferences = Preferences::default();

        client
            .execute(
                "UPDATE users SET preferences = $1 WHERE id = $2",
                &[&Json(preferences), &user_id],
            )
            .await
            .unwrap();

        // Create a monthly goal
        let goal = Goal {
            id: None,
            user_id,
            name: "Test Goal with Preferences".to_string(),
            recurrence: Recurrence::Monthly,
            target: Decimal::new(300, 0),
            target_date: Utc::now() + Duration::days(15),
            accumulated_amount: Decimal::new(150, 0),
            start_date: None,
        };
        let mut goal = goal.create(&client).await.unwrap();
        // Set accumulated amount after creation since create() sets it to ZERO
        goal.accumulated_amount = Decimal::new(150, 0);
        let _goal = goal.update(&client).await.unwrap();

        let app = Router::new()
            .route("/goals/resets/{recurrence}", post(action))
            .layer(user_extension)
            .layer(context_extension)
            .with_state(shared_state.clone());

        let request = Request::builder()
            .method("POST")
            .uri("/goals/resets/monthly")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
    }

    #[tokio::test]
    async fn test_resets_action_non_monthly_goals_unchanged() {
        let (shared_state, user_extension, context_extension) = state_for_tests().await.unwrap();
        let client = shared_state.pool.get_client().await.unwrap();
        let user_id = user_extension.0.id;

        // Create goals with different recurrences
        let daily_goal = Goal {
            id: None,
            user_id,
            name: "Daily Goal".to_string(),
            recurrence: Recurrence::Daily,
            target: Decimal::new(50, 0),
            target_date: Utc::now() + Duration::days(1),
            accumulated_amount: Decimal::new(25, 0),
            start_date: None,
        };
        let mut daily_goal = daily_goal.create(&client).await.unwrap();
        // Set accumulated amount after creation since create() sets it to ZERO
        daily_goal.accumulated_amount = Decimal::new(25, 0);
        let daily_goal = daily_goal.update(&client).await.unwrap();

        let yearly_goal = Goal {
            id: None,
            user_id,
            name: "Yearly Goal".to_string(),
            recurrence: Recurrence::Yearly,
            target: Decimal::new(5000, 0),
            target_date: Utc::now() + Duration::days(365),
            accumulated_amount: Decimal::new(2500, 0),
            start_date: None,
        };
        let mut yearly_goal = yearly_goal.create(&client).await.unwrap();
        // Set accumulated amount after creation since create() sets it to ZERO
        yearly_goal.accumulated_amount = Decimal::new(2500, 0);
        let yearly_goal = yearly_goal.update(&client).await.unwrap();

        let app = Router::new()
            .route("/goals/resets/{recurrence}", post(action))
            .layer(user_extension)
            .layer(context_extension)
            .with_state(shared_state.clone());

        let request = Request::builder()
            .method("POST")
            .uri("/goals/resets/monthly")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::SEE_OTHER);

        // Verify non-monthly goals were not reset
        let updated_daily = Goal::get_one(&client, daily_goal.id.unwrap(), user_id)
            .await
            .unwrap();
        assert_eq!(updated_daily.accumulated_amount, Decimal::new(25, 0));

        let updated_yearly = Goal::get_one(&client, yearly_goal.id.unwrap(), user_id)
            .await
            .unwrap();
        assert_eq!(updated_yearly.accumulated_amount, Decimal::new(2500, 0));
    }

    #[tokio::test]
    async fn test_resets_action_no_goals() {
        let (shared_state, user_extension, context_extension) = state_for_tests().await.unwrap();

        let app = Router::new()
            .route("/goals/resets/{recurrence}", post(action))
            .layer(user_extension)
            .layer(context_extension)
            .with_state(shared_state.clone());

        let request = Request::builder()
            .method("POST")
            .uri("/goals/resets/monthly")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
    }
}
