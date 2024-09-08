use super::GoalForm;
use crate::{
    authenticated::UserExtension,
    errors::FormError,
    models::goal::{Goal, Recurrence},
    SharedState,
};
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    Extension, Form,
};
use bson::oid::ObjectId;
use chrono::{NaiveDateTime, NaiveTime};
use mongodb::Collection;
use std::str::FromStr;
use tera::Context;
use validator::Validate;

pub async fn page(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
    headers: HeaderMap,
    form: Form<GoalForm>,
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
            context.insert("name", &form.name);
            context.insert("target", &form.target);
            context.insert("target_date", &form.target_date);
            context.insert("recurrence", &form.recurrence);

            let content = shared_state
                .tera
                .render(
                    if turbo {
                        "goals/form.turbo.html"
                    } else {
                        "goals/form.html"
                    },
                    &context,
                )
                .unwrap();

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

    let goal_record = Goal {
        _id: ObjectId::new().to_string(),
        name: form.name.to_owned(),
        target: form.target.to_owned(),
        recurrence: Recurrence::from_str(&form.recurrence).unwrap(),
        target_date: NaiveDateTime::new(form.target_date, NaiveTime::MIN).and_utc(),
        user_id: ObjectId::from_str(&user.id).unwrap().to_string(),
    };

    let goals: Collection<Goal> = shared_state
        .mongo
        .default_database()
        .unwrap()
        .collection("goals");

    let _ = goals.insert_one(goal_record).await?;

    Ok(Redirect::to("/goals").into_response())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::test_utils::{state_for_tests, user_for_tests};
    use axum::body::{to_bytes, Body};
    use axum::http::{Request, StatusCode};
    use axum::routing::post;
    use axum::Router;
    use bson::doc;
    use chrono::{Duration, Utc};
    use std::ops::Add;
    use std::str::from_utf8;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_create_goal_success() {
        let shared_state = state_for_tests().await;
        let goals: Collection<Goal> = shared_state
            .mongo
            .default_database()
            .unwrap()
            .collection("goals");

        goals
            .delete_one(doc! {"name": "test_create_goal_success"})
            .await
            .unwrap();

        let app = Router::new()
            .route("/goals/create", post(page))
            .with_state(shared_state)
            .layer(user_for_tests(&ObjectId::new().to_hex()));

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

        // Verify that the goal was created in the database
        let goal = goals
            .find_one(doc! {"name": "test_create_goal_success"})
            .await
            .unwrap();

        assert!(goal.is_some())
    }

    #[tokio::test]
    async fn test_create_goal_validation_error() {
        let shared_state = state_for_tests().await;
        let app = Router::new()
            .route("/goals/create", post(page))
            .with_state(shared_state)
            .layer(user_for_tests(&ObjectId::new().to_hex()));

        let form_data = "name=test&target=124&target_date=2024-09-13&recurrence=monthly";
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
        let shared_state = state_for_tests().await;
        let app = Router::new()
            .route("/goals/create", post(page))
            .layer(user_for_tests(&ObjectId::new().to_hex()))
            .with_state(shared_state);

        let form_data = "name=test&target=124&target_date=2024-09-13&recurrence=monthly";
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
