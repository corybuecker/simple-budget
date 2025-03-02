use crate::{SharedState, authenticated::UserExtension, errors::FormError, models::goal::Goal};
use axum::{
    Extension,
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use tera::Context;

pub async fn modal(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
    Path(id): Path<i32>,
) -> Result<Response, FormError> {
    let goal = Goal::get_one(&shared_state.client, id, user.id).await?;
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
) -> Result<Response, FormError> {
    let goal = Goal::get_one(&shared_state.client, id, user.id).await?;

    goal.delete(&shared_state.client).await?;
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

//#[cfg(test)]
//mod tests {
//    use crate::models::goal::Goal;
//    use crate::test_utils::{state_for_tests, user_for_tests};
//    use axum::Router;
//    use axum::body::Body;
//    use axum::http::{Request, StatusCode};
//    use bson::doc;
//    use bson::oid::ObjectId;
//    use chrono::Duration;
//    use std::ops::Add;
//    use tower::ServiceExt;
//
//    #[tokio::test]
//    async fn test_delete_action() {
//        let shared_state = state_for_tests().await;
//        let goals = shared_state
//            .mongo
//            .default_database()
//            .unwrap()
//            .collection::<Goal>("goals");
//
//        goals
//            .delete_many(doc! {"name": "delete_goals"})
//            .await
//            .unwrap();
//
//        let user_id = ObjectId::new();
//        let goal_id = ObjectId::new();
//
//        let goal = Goal {
//            _id: goal_id.to_string(),
//            user_id: user_id.to_string(),
//            name: "delete_goal".to_string(),
//            target: 100.0,
//            recurrence: crate::models::goal::Recurrence::Weekly,
//            target_date: chrono::Utc::now().add(Duration::seconds(60)),
//        };
//
//        goals.insert_one(goal).await.unwrap();
//
//        // Create a router with the delete route
//        let app = Router::new()
//            .route("/goals/{id}", axum::routing::delete(super::action))
//            .layer(user_for_tests(&user_id.to_hex()))
//            .with_state(shared_state);
//
//        let request = Request::builder()
//            .uri(format!("/goals/{}", goal_id))
//            .method("DELETE")
//            .body(Body::empty())
//            .unwrap();
//
//        let response = app.oneshot(request).await.unwrap();
//
//        assert_eq!(response.status(), StatusCode::OK);
//
//        let deleted_goal = goals.find_one(doc! {"_id": goal_id}).await.unwrap();
//        assert!(deleted_goal.is_none());
//    }
//}
