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
use bson::{doc, oid::ObjectId};
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
    match accept {
        Some(accept) => {
            if accept.to_str().unwrap().contains("turbo") {
                turbo = true;
            }
        }
        _ => {}
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

    let goals: mongodb::Collection<Goal> = shared_state
        .mongo
        .default_database()
        .unwrap()
        .collection("goals");

    let filter = doc! {"_id": ObjectId::from_str(&id).unwrap(), "user_id": ObjectId::from_str(&user.id).unwrap()};
    log::debug!("{:?}", filter);

    let goal = goals.find_one(filter.clone()).await?;

    let Some(mut goal) = goal else {
        return Err(FormError {
            message: "could not find goal".to_string(),
            status_code: Some(StatusCode::NOT_FOUND),
        });
    };

    goal.name = form.name.clone();
    goal.target = form.target;
    goal.target_date = NaiveDateTime::new(form.target_date, NaiveTime::MIN).and_utc();
    goal.recurrence = Recurrence::from_str(&form.recurrence).unwrap();
    let _ = goals.replace_one(filter, goal).await?;

    Ok(Redirect::to("/goals").into_response())
}

#[cfg(test)]
mod tests {
    use std::ops::Add;

    use crate::{authenticated::UserExtension, models::goal::Goal, mongo_client, SharedState};
    use axum::{
        http::{Method, Request, StatusCode},
        Extension,
    };
    use axum_extra::extract::cookie::Key;
    use chrono::Duration;
    use mongodb::bson::doc;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_update_goal() {
        // Set up the database connection
        let client = mongo_client().await.unwrap();
        let db = client.default_database().unwrap();
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

        // Set up the SharedState
        let shared_state = SharedState {
            mongo: client,
            key: Key::generate(),
            tera: tera::Tera::new("templates/**/*").unwrap(),
        };

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
                "/goals/:id",
                axum::routing::post(crate::authenticated::goals::update::action),
            )
            .with_state(shared_state)
            .layer(Extension(UserExtension {
                id: user_id.to_string(),
                csrf: "test".to_string(),
            }));

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
