use crate::{
    HandlebarsContext, SharedState,
    authenticated::UserExtension,
    errors::AppResponse,
    models::{
        goal::{Goal, Recurrence},
        user::{GoalHeader, User},
    },
    utilities::responses::{self, ResponseFormat, generate_response},
};
use anyhow::anyhow;
use axum::{
    Extension,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Redirect},
};
use chrono::Utc;
use handlebars::to_json;
use rust_decimal::Decimal;
use std::collections::HashMap;
use tokio_postgres::GenericClient;
use tracing::debug;

pub async fn action(
    shared_state: State<SharedState>,
    Path(_recurrence): Path<String>,
    headers: HeaderMap,
    Extension(user): Extension<UserExtension>,
    Extension(context): Extension<HandlebarsContext>,
) -> AppResponse {
    let mut context = context.clone();
    let response_format = responses::get_response_format(&headers)?;
    let client = shared_state.pool.get().await?;
    let client = client.client();
    let user_id = user.id;

    let mut goals = Goal::get_all(client, user_id).await?;
    let mut accumulations: HashMap<i32, Decimal> = HashMap::new();
    let mut days_remainings: HashMap<i32, i16> = HashMap::new();
    let mut per_days: HashMap<i32, Decimal> = HashMap::new();

    for goal in &mut goals {
        if goal.recurrence.eq(&Recurrence::Monthly) {
            goal.accumulated_amount = Decimal::ZERO;
            goal.update(client).await?;
        }

        accumulations.insert(goal.id.unwrap(), goal.accumulated_amount);
        per_days.insert(goal.id.unwrap(), goal.accumulated_per_day()?);
        days_remainings.insert(
            goal.id.unwrap(),
            (goal.target_date - Utc::now())
                .num_days()
                .try_into()
                .unwrap(),
        );
    }

    match response_format {
        ResponseFormat::Html => Ok(Redirect::to("/goals").into_response()),
        ResponseFormat::Turbo => {
            let user = User::get_by_id(client, user_id).await?;
            let goal_header = match user.preferences {
                Some(preferences) => preferences.0.goal_header,
                None => Some(GoalHeader::Accumulated),
            };

            // Use a cloned value for the context to avoid the move issue
            let goal_header_for_context = goal_header.clone();
            context.insert(
                "goal_header".to_string(),
                to_json(goal_header_for_context.or(Some(GoalHeader::Accumulated))),
            );

            context.insert("goals".to_string(), to_json(&goals));
            context.insert("accumulations".to_string(), to_json(&accumulations));
            context.insert("days_remainings".to_string(), to_json(&days_remainings));
            context.insert("per_days".to_string(), to_json(&per_days));

            let content = shared_state
                .handlebars
                .render("goals/resets.turbo", &context);

            debug!("{:#?}", content);

            Ok(generate_response(
                &response_format,
                content?,
                StatusCode::OK,
            ))
        }
        ResponseFormat::Json => Err(anyhow!("Not implemented").into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::goal::{Goal, Recurrence};
    use crate::models::user::{GoalHeader, Preferences};
    use crate::test_utils::state_for_tests;
    use axum::Router;
    use axum::body::Body;
    use axum::http::{Request, StatusCode, header};
    use axum::routing::post;
    use chrono::{Duration, Utc};
    use postgres_types::Json;
    use rust_decimal::Decimal;
    use tokio_postgres::GenericClient;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_resets_action_html_redirect() {
        let (shared_state, user_extension, context_extension) = state_for_tests().await.unwrap();
        let client = shared_state.pool.get().await.unwrap();
        let client = client.client();
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
        let mut goal = goal.create(client).await.unwrap();
        // Set accumulated amount after creation since create() sets it to ZERO
        goal.accumulated_amount = Decimal::new(500, 0);
        let goal = goal.update(client).await.unwrap();

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
        let updated_goal = Goal::get_one(client, goal.id.unwrap(), user_id)
            .await
            .unwrap();
        assert_eq!(updated_goal.accumulated_amount, Decimal::ZERO);
    }

    #[tokio::test]
    async fn test_resets_action_turbo_response() {
        let (shared_state, user_extension, context_extension) = state_for_tests().await.unwrap();
        let client = shared_state.pool.get().await.unwrap();
        let client = client.client();
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
        let mut monthly_goal = monthly_goal.create(client).await.unwrap();
        // Set accumulated amount after creation since create() sets it to ZERO
        monthly_goal.accumulated_amount = Decimal::new(750, 0);
        let monthly_goal = monthly_goal.update(client).await.unwrap();

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
        let mut weekly_goal = weekly_goal.create(client).await.unwrap();
        // Set accumulated amount after creation since create() sets it to ZERO
        weekly_goal.accumulated_amount = Decimal::new(100, 0);
        let weekly_goal = weekly_goal.update(client).await.unwrap();

        let app = Router::new()
            .route("/goals/resets/{recurrence}", post(action))
            .layer(user_extension)
            .layer(context_extension)
            .with_state(shared_state.clone());

        let request = Request::builder()
            .method("POST")
            .uri("/goals/resets/monthly")
            .header(header::ACCEPT, "text/vnd.turbo-stream.html")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "text/vnd.turbo-stream.html"
        );

        // Verify only monthly goals were reset
        let updated_monthly = Goal::get_one(client, monthly_goal.id.unwrap(), user_id)
            .await
            .unwrap();
        assert_eq!(updated_monthly.accumulated_amount, Decimal::ZERO);

        let updated_weekly = Goal::get_one(client, weekly_goal.id.unwrap(), user_id)
            .await
            .unwrap();
        assert_eq!(updated_weekly.accumulated_amount, Decimal::new(100, 0)); // Should not be reset
    }

    #[tokio::test]
    async fn test_resets_action_with_user_preferences() {
        let (shared_state, user_extension, context_extension) = state_for_tests().await.unwrap();
        let client = shared_state.pool.get().await.unwrap();
        let client = client.client();
        let user_id = user_extension.0.id;

        // Update user preferences
        let preferences = Preferences {
            goal_header: Some(GoalHeader::PerDay),
            timezone: None,
            forecast_offset: None,
            monthly_income: None,
        };

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
        let mut goal = goal.create(client).await.unwrap();
        // Set accumulated amount after creation since create() sets it to ZERO
        goal.accumulated_amount = Decimal::new(150, 0);
        let _goal = goal.update(client).await.unwrap();

        let app = Router::new()
            .route("/goals/resets/{recurrence}", post(action))
            .layer(user_extension)
            .layer(context_extension)
            .with_state(shared_state.clone());

        let request = Request::builder()
            .method("POST")
            .uri("/goals/resets/monthly")
            .header(header::ACCEPT, "text/vnd.turbo-stream.html")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "text/vnd.turbo-stream.html"
        );
    }

    #[tokio::test]
    async fn test_resets_action_non_monthly_goals_unchanged() {
        let (shared_state, user_extension, context_extension) = state_for_tests().await.unwrap();
        let client = shared_state.pool.get().await.unwrap();
        let client = client.client();
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
        let mut daily_goal = daily_goal.create(client).await.unwrap();
        // Set accumulated amount after creation since create() sets it to ZERO
        daily_goal.accumulated_amount = Decimal::new(25, 0);
        let daily_goal = daily_goal.update(client).await.unwrap();

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
        let mut yearly_goal = yearly_goal.create(client).await.unwrap();
        // Set accumulated amount after creation since create() sets it to ZERO
        yearly_goal.accumulated_amount = Decimal::new(2500, 0);
        let yearly_goal = yearly_goal.update(client).await.unwrap();

        let app = Router::new()
            .route("/goals/resets/{recurrence}", post(action))
            .layer(user_extension)
            .layer(context_extension)
            .with_state(shared_state.clone());

        let request = Request::builder()
            .method("POST")
            .uri("/goals/resets/monthly")
            .header(header::ACCEPT, "text/vnd.turbo-stream.html")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Verify non-monthly goals were not reset
        let updated_daily = Goal::get_one(client, daily_goal.id.unwrap(), user_id)
            .await
            .unwrap();
        assert_eq!(updated_daily.accumulated_amount, Decimal::new(25, 0));

        let updated_yearly = Goal::get_one(client, yearly_goal.id.unwrap(), user_id)
            .await
            .unwrap();
        assert_eq!(updated_yearly.accumulated_amount, Decimal::new(2500, 0));
    }

    #[tokio::test]
    async fn test_resets_action_json_not_implemented() {
        let (shared_state, user_extension, context_extension) = state_for_tests().await.unwrap();

        let app = Router::new()
            .route("/goals/resets/{recurrence}", post(action))
            .layer(user_extension)
            .layer(context_extension)
            .with_state(shared_state.clone());

        let request = Request::builder()
            .method("POST")
            .uri("/goals/resets/monthly")
            .header(header::ACCEPT, "application/json")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
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
            .header(header::ACCEPT, "text/vnd.turbo-stream.html")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "text/vnd.turbo-stream.html"
        );
    }
}
