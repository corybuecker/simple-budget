use crate::{SharedState, authenticated::UserExtension, errors::AppResponse, models::goal::Goal};
use axum::{
    Extension,
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse},
};
use tera::Context;
use tokio_postgres::GenericClient;

pub async fn modal(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
    Path(id): Path<i32>,
) -> AppResponse {
    let goal = Goal::get_one(shared_state.pool.get().await?.client(), id, user.id).await?;
    let tera = shared_state.tera.clone();
    let mut context = Context::new();
    context.insert("goal", &goal);
    let content = tera.render("goals/delete/confirm.html", &context)?;

    Ok(Html::from(content).into_response())
}

pub async fn action(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
    Path(id): Path<i32>,
) -> AppResponse {
    let goal = Goal::get_one(shared_state.pool.get().await?.client(), id, user.id).await?;

    goal.delete(shared_state.pool.get().await?.client()).await?;
    let tera = shared_state.tera.clone();
    let mut context = Context::new();
    context.insert("goal", &goal);
    let content = tera.render("goals/delete.html", &context)?;

    Ok((
        StatusCode::OK,
        [("content-type", "text/vnd.turbo-stream.html")],
        Html::from(content),
    )
        .into_response())
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
    use tokio_postgres::GenericClient;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_delete_action() {
        let (shared_state, user_extension) = state_for_tests().await.unwrap();
        let user_id = user_extension.0.id;
        let goal = Goal {
            id: None,
            user_id: user_extension.0.id,
            recurrence: Recurrence::Weekly,
            name: "Test Goal".to_string(),
            target: Decimal::new(1000, 0),
            target_date: Utc::now(),
            accumulated_amount: Decimal::ZERO,
        };

        let goal = goal
            .create(shared_state.pool.get().await.unwrap().client())
            .await
            .unwrap();

        let app = Router::new()
            .route("/goals/{id}", axum::routing::delete(action))
            .layer(user_extension)
            .with_state(shared_state.clone());

        let request = Request::builder()
            .uri(format!("/goals/{}", goal.id.unwrap()))
            .method("DELETE")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let deleted_goal = Goal::get_one(
            shared_state.pool.get().await.unwrap().client(),
            goal.id.unwrap(),
            user_id,
        )
        .await;
        assert!(deleted_goal.is_err());
    }
}
