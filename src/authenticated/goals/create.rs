use super::{GoalForm, schema};
use crate::{
    SharedState,
    authenticated::UserExtension,
    errors::AppResponse,
    models::goal::{Goal, Recurrence},
    utilities::responses,
};
use anyhow::anyhow;
use axum::{
    Extension, Form,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Redirect},
};
use chrono::{NaiveDateTime, NaiveTime};
use rust_decimal::{Decimal, prelude::FromPrimitive};
use std::str::FromStr;
use tera::Context;
use tokio_postgres::GenericClient;

pub async fn page(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
    headers: HeaderMap,
    Form(form): Form<GoalForm>,
) -> AppResponse {
    let json = serde_json::to_value(&form)?;
    let valid = jsonschema::validate(&schema(), &json);
    let response_format = responses::get_response_format(&headers)?;

    if valid.is_err() {
        let validation_errors = valid.unwrap_err();
        let mut context = Context::new();

        context.insert("errors", &validation_errors.to_string());
        context.insert("name", &form.name);
        context.insert("target", &form.target);
        context.insert("target_date", &form.target_date);
        context.insert("recurrence", &form.recurrence);

        let template_name = responses::get_template_name(&response_format, "goals", "form");
        let content = shared_state.tera.render(&template_name, &context)?;

        return Ok(responses::generate_response(
            &response_format,
            content,
            StatusCode::BAD_REQUEST,
        ));
    }

    let goal = Goal {
        id: None,
        name: form.name.to_owned(),
        target: Decimal::from_f64(form.target.to_owned())
            .ok_or_else(|| anyhow!("could not parse decimal"))?,
        recurrence: Recurrence::from_str(&form.recurrence).map_err(|e| anyhow!("{:#?}", e))?,
        target_date: NaiveDateTime::new(form.target_date, NaiveTime::MIN).and_utc(),
        user_id: user.id,
        accumulated_amount: Decimal::ZERO,
    };

    goal.create(shared_state.pool.get().await?.client()).await?;

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
    use std::ops::Add;
    use std::str::from_utf8;
    use tokio_postgres::GenericClient;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_create_goal_success() {
        let (shared_state, user_extension, _context_extension) = state_for_tests().await.unwrap();
        let client = shared_state.pool.get().await.unwrap();
        let client = client.client();
        let user_id = user_extension.0.id;

        let app = Router::new()
            .route("/goals/create", post(page))
            .with_state(shared_state.clone())
            .layer(user_extension);

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
        let (shared_state, user_extension, _context_extension) = state_for_tests().await.unwrap();

        let app = Router::new()
            .route("/goals/create", post(page))
            .with_state(shared_state)
            .layer(user_extension);

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
        assert!(body_str.contains("test"))
    }

    #[tokio::test]
    async fn test_create_goal_turbo_stream() {
        let (shared_state, user_extension, _context_extension) = state_for_tests().await.unwrap();
        let app = Router::new()
            .route("/goals/create", post(page))
            .with_state(shared_state.clone())
            .layer(user_extension);

        let form_data = "name=t&target=124&target_date=2024-09-13&recurrence=monthly";
        let request = Request::builder()
            .method("POST")
            .uri("/goals/create")
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
