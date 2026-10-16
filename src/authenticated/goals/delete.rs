use crate::{
    LayoutCtx, SharedState, authenticated::UserExtension, errors::AppResponse, models::goal::Goal,
    templates::DeleteConfirmation, utilities::responses::generate_response,
};
use askama::Template;
use axum::{
    Extension,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use uuid::Uuid;

pub async fn modal(
    shared_state: State<SharedState>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<UserExtension>,
    Extension(layout): Extension<LayoutCtx>,
) -> AppResponse {
    let client = shared_state.pool.get_client().await?;
    let goal = Goal::get_one(&client, id, user.id).await?;

    let template = DeleteConfirmation {
        layout,
        prompt: "Are you sure you want to delete this goal?".to_string(),
        action: format!("/goals/{}/delete", id),
        cancel: "/goals".to_string(),
        entity: goal.name,
    };

    Ok(generate_response(template.render()?, StatusCode::OK))
}

pub async fn action(
    shared_state: State<SharedState>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<UserExtension>,
) -> AppResponse {
    let client = shared_state.pool.get_client().await?;
    let goal = Goal::get_one(&client, id, user.id).await?;

    goal.delete(&client).await?;

    Ok(Redirect::to("/goals").into_response())
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::models::goal::{Goal, Recurrence};
    use crate::test_utils::state_for_tests;
    use axum::Router;
    use axum::body::Body;
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
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers().get("location").unwrap(), "/goals");

        let deleted_goal = Goal::get_one(&client, goal.id.unwrap(), user_id).await;
        assert!(deleted_goal.is_err());
    }
}
