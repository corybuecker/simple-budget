use super::{GoalForm, NewTemplate, schema};
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
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use chrono::{NaiveDateTime, NaiveTime, Utc};
use rust_decimal::{Decimal, prelude::FromPrimitive};
use std::str::FromStr;

pub async fn action(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
    Extension(layout): Extension<LayoutCtx>,
    Form(form): Form<GoalForm>,
) -> AppResponse {
    let json = serde_json::to_value(&form)?;

    if let Err(validation_errors) = jsonschema::validate(&schema(), &json) {
        let template = NewTemplate {
            layout,
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

    let recurrence = Recurrence::from_str(&form.recurrence).map_err(|e| anyhow!("{:#?}", e))?;
    let start_date = match recurrence {
        Recurrence::Never => Some(Utc::now()),
        _ => None,
    };

    let goal = Goal {
        id: None,
        name: form.name.to_owned(),
        target: Decimal::from_f64(form.target.to_owned())
            .ok_or_else(|| anyhow!("could not parse decimal"))?,
        target_date: NaiveDateTime::new(form.target_date, NaiveTime::MIN).and_utc(),
        user_id: user.id,
        accumulated_amount: Decimal::ZERO,
        recurrence,
        start_date,
    };
    let client = shared_state.pool.get_client().await?;
    goal.create(&client).await?;

    Ok(Redirect::to("/goals").into_response())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::state_for_tests;
    use axum::Router;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use axum::routing::post;
    use chrono::{Duration, Utc};
    use rust_database_common::GenericClient;
    use std::ops::Add;
    use std::str::from_utf8;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_create_goal_success() {
        let (shared_state, user_extension, context_extension) = state_for_tests().await.unwrap();
        let client = shared_state.pool.get_client().await.unwrap();
        let user_id = user_extension.0.id;

        let app = Router::new()
            .route("/goals/create", post(action))
            .with_state(shared_state.clone())
            .layer(user_extension)
            .layer(context_extension);

        let target_date = Utc::now().add(Duration::days(7));

        let form_data = format!(
            "name=test_create_goal_success&target=124&target_date={}&recurrence=monthly",
            target_date.format("%Y-%m-%d")
        );
        let request = Request::builder()
            .method("POST")
            .uri("/goals/create")
            .header("content-type", "application/x-www-form-urlencoded")
            .body(Body::from(form_data))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers().get("location").unwrap(), "/goals");

        let goal = client
            .query_one(
                "SELECT * FROM goals WHERE user_id = $1 LIMIT 1",
                &[&user_id],
            )
            .await
            .unwrap();

        assert_eq!(goal.get::<_, String>("name"), "test_create_goal_success")
    }

    #[tokio::test]
    async fn test_create_goal_validation_error() {
        let (shared_state, user_extension, context_extension) = state_for_tests().await.unwrap();

        let app = Router::new()
            .route("/goals/create", post(action))
            .with_state(shared_state)
            .layer(user_extension)
            .layer(context_extension);

        let form_data = "name=t&target=124&target_date=2024-09-13&recurrence=monthly";
        let request = Request::builder()
            .method("POST")
            .uri("/goals/create")
            .header("content-type", "application/x-www-form-urlencoded")
            .body(Body::from(form_data))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        let (parts, body) = response.into_parts();
        let bytes = to_bytes(body, usize::MAX).await.unwrap();
        let body_str = from_utf8(&bytes).unwrap().to_string();

        assert_eq!(parts.status, StatusCode::BAD_REQUEST);
        assert!(body_str.contains("124"))
    }

    #[tokio::test]
    async fn test_create_goal_with_explicit_start_date_never_recurrence() {
        let (shared_state, user_extension, context_extension) = state_for_tests().await.unwrap();
        let client = shared_state.pool.get_client().await.unwrap();
        let user_id = user_extension.0.id;

        let app = Router::new()
            .route("/goals/create", post(action))
            .with_state(shared_state.clone())
            .layer(user_extension)
            .layer(context_extension);

        let target_date = Utc::now().add(Duration::days(7));

        let form_data = format!(
            "name=test_create_goal_success&target=124&target_date={}&recurrence=never",
            target_date.format("%Y-%m-%d")
        );
        let request = Request::builder()
            .method("POST")
            .uri("/goals/create")
            .header("content-type", "application/x-www-form-urlencoded")
            .body(Body::from(form_data))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers().get("location").unwrap(), "/goals");

        let goal = client
            .query_one(
                "SELECT * FROM goals WHERE user_id = $1 LIMIT 1",
                &[&user_id],
            )
            .await
            .unwrap();

        assert_eq!(goal.get::<_, String>("name"), "test_create_goal_success");
        let start_date_str = goal
            .get::<_, chrono::DateTime<chrono::Utc>>("start_date")
            .to_rfc3339();
        assert!(chrono::DateTime::parse_from_rfc3339(&start_date_str).is_ok());
    }
}
