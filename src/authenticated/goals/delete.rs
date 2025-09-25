use crate::{
    HandlebarsContext, SharedState,
    authenticated::UserExtension,
    errors::AppResponse,
    models::goal::Goal,
    utilities::responses::{ResponseFormat, generate_response, get_response_format},
};
use axum::{
    Extension, Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use handlebars::to_json;

pub async fn modal(
    shared_state: State<SharedState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
    Extension(user): Extension<UserExtension>,
    Extension(context): Extension<HandlebarsContext>,
) -> AppResponse {
    let client = shared_state.pool.get_client().await?;
    let goal = Goal::get_one(&client, id, user.id).await?;
    let response_format = get_response_format(&headers)?;

    match response_format {
        ResponseFormat::Html => {
            let mut context = context.clone();
            context.insert(
                "prompt".to_string(),
                to_json("Are you sure you want to delete this goal?"),
            );
            context.insert("action".to_string(), to_json(format!("/goals/{}", id)));
            context.insert("entity".to_string(), to_json(goal.name));
            context.insert("partial".to_string(), to_json("delete_confirmation"));
            Ok(generate_response(
                &response_format,
                shared_state
                    .handlebars
                    .render("delete_confirmation", &context)?,
                StatusCode::OK,
            ))
        }
        ResponseFormat::Turbo => Ok(StatusCode::NOT_ACCEPTABLE.into_response()),
        ResponseFormat::Json => Ok(generate_response(
            &response_format,
            Json(goal),
            StatusCode::OK,
        )),
    }
}

pub async fn action(
    shared_state: State<SharedState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
    Extension(user): Extension<UserExtension>,
    Extension(context): Extension<HandlebarsContext>,
) -> AppResponse {
    let client = shared_state.pool.get_client().await?;
    let goal = Goal::get_one(&client, id, user.id).await?;
    goal.delete(&client).await?;
    let response_format = get_response_format(&headers)?;

    match response_format {
        ResponseFormat::Html => Ok(StatusCode::NOT_ACCEPTABLE.into_response()),
        ResponseFormat::Json => Ok(generate_response(
            &response_format,
            Json(goal),
            StatusCode::OK,
        )),
        ResponseFormat::Turbo => {
            let mut context = context.clone();
            context.insert("goal".to_string(), to_json(&goal));

            Ok(generate_response(
                &response_format,
                shared_state.handlebars.render("goals/delete", &context)?,
                StatusCode::OK,
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::from_utf8;

    use super::*;
    use crate::models::goal::{Goal, Recurrence};
    use crate::test_utils::state_for_tests;
    use axum::Router;
    use axum::body::{Body, to_bytes};
    use axum::http::Request;
    use chrono::Utc;
    use rust_decimal::Decimal;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_delete_modal() {
        let (shared_state, user_extension, context_extension) = state_for_tests().await.unwrap();
        let goal = Goal {
            id: None,
            user_id: user_extension.0.id,
            recurrence: Recurrence::Weekly,
            name: "Test Goal".to_string(),
            target: Decimal::new(1000, 0),
            target_date: Utc::now(),
            accumulated_amount: Decimal::ZERO,
            start_date: None,
        };
        let client = shared_state.pool.get_client().await.unwrap();

        let goal = goal.create(&client).await.unwrap();

        let app = Router::new()
            .route("/goals/{id}/delete", axum::routing::get(modal))
            .layer(user_extension)
            .layer(context_extension)
            .with_state(shared_state.clone());

        let request = Request::builder()
            .uri(format!("/goals/{}/delete", goal.id.unwrap()))
            .method("GET")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_delete_action() {
        let (shared_state, user_extension, context_extension) = state_for_tests().await.unwrap();
        let user_id = user_extension.0.id;
        let goal = Goal {
            id: None,
            user_id: user_extension.0.id,
            recurrence: Recurrence::Weekly,
            name: "Test Goal".to_string(),
            target: Decimal::new(1000, 0),
            target_date: Utc::now(),
            accumulated_amount: Decimal::ZERO,
            start_date: None,
        };

        let client = shared_state.pool.get_client().await.unwrap();

        let goal = goal.create(&client).await.unwrap();

        let app = Router::new()
            .route("/goals/{id}", axum::routing::delete(action))
            .layer(user_extension)
            .layer(context_extension)
            .with_state(shared_state.clone());

        let request = Request::builder()
            .uri(format!("/goals/{}", goal.id.unwrap()))
            .method("DELETE")
            .header("Accept", "turbo")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        let (parts, body) = response.into_parts();
        let bytes = to_bytes(body, usize::MAX).await.unwrap();
        let body_str = from_utf8(&bytes).unwrap().to_string();
        println!("{:?}", body_str);
        assert_eq!(parts.status, StatusCode::OK);

        let deleted_goal = Goal::get_one(&client, goal.id.unwrap(), user_id).await;
        assert!(deleted_goal.is_err());
    }
}
