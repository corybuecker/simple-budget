use crate::{authenticated::UserExtension, models::goal::Goal, SharedState};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    Extension,
};
use bson::{doc, oid::ObjectId};
use std::str::FromStr;

pub async fn action(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
    Path(id): Path<String>,
) -> Result<Response, StatusCode> {
    let goals: mongodb::Collection<Goal> = shared_state
        .mongo
        .default_database()
        .unwrap()
        .collection("goals");

    let filter = doc! {"_id": ObjectId::from_str(&id).unwrap(), "user_id": ObjectId::from_str(&user.id).unwrap()};

    let Ok(goal) = goals.find_one(filter.clone()).await else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    let Some(_) = goal else {
        return Err(StatusCode::NOT_FOUND);
    };

    let _ = goals.delete_one(filter).await;

    Ok(Redirect::to("/goals").into_response())
}

#[cfg(test)]
mod tests {
    use std::ops::Add;

    use super::*;
    use crate::models::goal::Goal;
    use crate::mongo_client;
    use axum::body::Body;
    use axum::http::Request;
    use axum::Router;
    use axum_extra::extract::cookie::Key;
    use chrono::Duration;
    use tera::Tera;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_delete_action() {
        let client = mongo_client().await.unwrap();
        let goals = client
            .default_database()
            .unwrap()
            .collection::<Goal>("goals");

        goals
            .delete_many(doc! {"name": "delete_goals"})
            .await
            .unwrap();

        let user_id = ObjectId::new();
        let goal_id = ObjectId::new();

        let goal = Goal {
            _id: goal_id.to_string(),
            user_id: user_id.to_string(),
            name: "delete_goal".to_string(),
            target: 100.0,
            recurrence: crate::models::goal::Recurrence::Weekly,
            target_date: chrono::Utc::now().add(Duration::seconds(60)),
        };

        goals.insert_one(goal).await.unwrap();

        let tera = Tera::new("src/templates/**/*.html").expect("cannot initialize Tera");
        let shared_state = SharedState {
            mongo: client,
            key: Key::generate(),
            tera,
        };

        let user = UserExtension {
            id: user_id.to_string(),
            csrf: "test".to_string(),
        };

        // Create a router with the delete route
        let app = Router::new()
            .route("/goals/:id", axum::routing::delete(action))
            .layer(Extension(user))
            .with_state(shared_state);

        let request = Request::builder()
            .uri(format!("/goals/{}", goal_id.to_string()))
            .method("DELETE")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::SEE_OTHER);

        let deleted_goal = goals.find_one(doc! {"_id": goal_id}).await.unwrap();
        assert!(deleted_goal.is_none());
    }
}
