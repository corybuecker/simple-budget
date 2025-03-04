use super::GoalForm;
use crate::{
    authenticated::UserExtension,
    errors::FormError,
    models::goal::{Goal, Recurrence},
    SharedState,
};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    Extension, Form,
};
use bson::oid::ObjectId;
use chrono::{NaiveDateTime, NaiveTime};
use std::str::FromStr;
use tera::Context;
use validator::Validate;

pub async fn action(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
    Path(id): Path<String>,
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
            context.insert("id", &id);
            context.insert("name", &form.name);
            context.insert("target", &form.target);
            context.insert("target_date", &form.target_date);
            context.insert("recurrence", &form.recurrence);

            let content = shared_state.tera.render(
                if turbo {
                    "goals/form.turbo.html"
                } else {
                    "goals/edit.html"
                },
                &context,
            )?;

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
        _id: ObjectId::from_str(&id)?.to_string(),
        name: form.name.to_owned(),
        target: form.target.to_owned(),
        recurrence: Recurrence::from_str(&form.recurrence).unwrap(),
        target_date: NaiveDateTime::new(form.target_date, NaiveTime::MIN).and_utc(),
        user_id: ObjectId::from_str(&user.id)?.to_string(),
    };

    goal_record.update(&shared_state.mongo).await?;

    Ok(Redirect::to("/goals").into_response())
}

#[cfg(test)]
mod tests {
    use crate::{
        models::goal::Goal,
        test_utils::{self, user_for_tests},
    };
    use axum::http::{Method, Request, StatusCode};
    use chrono::Duration;
    use mongodb::bson::doc;
    use std::ops::Add;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_update_goal() {
        let shared_state = test_utils::state_for_tests().await;
        let db = shared_state.mongo.default_database().unwrap();
        let goals_collection: mongodb::Collection<Goal> = db.collection("goals");

        // Create a test goal
        let user_id = mongodb::bson::oid::ObjectId::new();
        let goal_id = mongodb::bson::oid::ObjectId::new();
        let test_goal = Goal {
            _id: goal_id.to_string(),
            user_id: user_id.to_string(),
            name: "update goal".to_string(),
            recurrence: crate::models::goal::Recurrence::Monthly,
            target: 100.0,
            target_date: chrono::Utc::now().add(Duration::seconds(2000)),
        };
        goals_collection.insert_one(test_goal).await.unwrap();

        let request = Request::builder()
            .method(Method::POST)
            .uri(format!("/goals/{}", goal_id.to_hex()))
            .header("content-type", "application/x-www-form-urlencoded")
            .body(
                "name=Updated%20Goal&target=2000.0&target_date=2024-12-31&recurrence=weekly"
                    .to_string(),
            )
            .unwrap();

        // Create a test app and call the action
        let app = axum::Router::new()
            .route(
                "/goals/{id}",
                axum::routing::post(crate::authenticated::goals::update::action),
            )
            .with_state(shared_state)
            .layer(user_for_tests(&user_id.to_string()));

        let response = app.oneshot(request).await.unwrap();

        // Assert the response
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers().get("location").unwrap(), "/goals");

        // Verify the goal was updated in the database
        let updated_goal = goals_collection
            .find_one(doc! {"_id": goal_id})
            .await
            .unwrap()
            .unwrap();

        assert_eq!(updated_goal.name, "Updated Goal");
        assert_eq!(updated_goal.target, 2000.0);
        assert_eq!(
            updated_goal.target_date,
            chrono::NaiveDate::from_ymd_opt(2024, 12, 31)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_utc()
        );

        // Clean up
        goals_collection
            .delete_one(doc! {"_id": goal_id})
            .await
            .unwrap();
    }
}
