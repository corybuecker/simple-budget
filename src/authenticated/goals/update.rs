use super::{GoalForm, schema};
use crate::{
    HandlebarsContext, SharedState,
    authenticated::UserExtension,
    errors::AppResponse,
    models::goal::{Goal, Recurrence},
    utilities::responses::{self, generate_response, get_response_format},
};
use anyhow::anyhow;
use axum::{
    Extension, Form, Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Redirect},
};
use chrono::{NaiveDateTime, NaiveTime};
use handlebars::to_json;
use rust_decimal::{Decimal, prelude::FromPrimitive};
use std::str::FromStr;

pub async fn action(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
    Path(id): Path<i32>,
    headers: HeaderMap,
    Extension(context): Extension<HandlebarsContext>,
    Form(form): Form<GoalForm>,
) -> AppResponse {
    let client = shared_state.pool.get_client().await?;
    let json = serde_json::to_value(&form)?;
    let valid = jsonschema::validate(&schema(), &json);

    let response_format = responses::get_response_format(&headers)?;

    match valid {
        Ok(_) => {}
        Err(validation_errors) => {
            let mut context = context.clone();

            context.insert("errors".to_string(), to_json(validation_errors.to_string()));
            context.insert("id".to_string(), to_json(id));
            context.insert("name".to_string(), to_json(&form.name));
            context.insert("target".to_string(), to_json(form.target));
            context.insert("target_date".to_string(), to_json(form.target_date));
            context.insert("recurrence".to_string(), to_json(&form.recurrence));

            match response_format {
                responses::ResponseFormat::Html => {
                    context.insert("partial".to_string(), to_json("goals/form"));
                    return Ok(responses::generate_response(
                        &responses::ResponseFormat::Html,
                        shared_state.handlebars.render("layout", &context)?,
                        StatusCode::BAD_REQUEST,
                    ));
                }
                responses::ResponseFormat::Turbo => {
                    return Ok(responses::generate_response(
                        &response_format,
                        shared_state.handlebars.render("goals/form", &context)?,
                        StatusCode::BAD_REQUEST,
                    ));
                }
                responses::ResponseFormat::Json => {
                    return Ok(responses::generate_response(
                        &response_format,
                        serde_json::to_string(&context)?,
                        StatusCode::BAD_REQUEST,
                    ));
                }
            }
        }
    }

    let mut goal = Goal::get_one(&client, id, user.id).await?;

    let new_recurrence = Recurrence::from_str(&form.recurrence).unwrap();

    goal.name = form.name.to_owned();
    goal.target = Decimal::from_f64(form.target.to_owned())
        .ok_or_else(|| anyhow!("could not parse decimal"))?;

    match new_recurrence {
        Recurrence::Never => match goal.recurrence {
            Recurrence::Never => {}
            _ => {
                goal.start_date = Some(chrono::Utc::now());
            }
        },
        _ => {
            goal.start_date = None;
        }
    }

    goal.recurrence = new_recurrence;
    goal.target_date = NaiveDateTime::new(form.target_date, NaiveTime::MIN).and_utc();
    goal.accumulated_amount = Decimal::ZERO;

    goal.update(&client).await?;

    match get_response_format(&headers)? {
        responses::ResponseFormat::Html | responses::ResponseFormat::Turbo => {
            Ok(Redirect::to("/goals").into_response())
        }
        responses::ResponseFormat::Json => Ok(generate_response(
            &responses::ResponseFormat::Json,
            Json(goal),
            StatusCode::OK,
        )),
    }
}

#[cfg(test)]
mod tests {
    use rust_database_common::GenericClient;
    #[tokio::test]
    async fn test_update_goal_recurrence_monthly_to_never_sets_start_date() {
        use crate::models::goal::{Goal, Recurrence};
        use chrono::{TimeZone, Utc};
        use rust_decimal::Decimal;
        let (shared_state, user_extension, context_extension) =
            crate::test_utils::state_for_tests().await.unwrap();
        let client = shared_state.pool.get_client().await.unwrap();
        let user_id = user_extension.0.id;

        // Create a monthly goal
        let mut goal = Goal {
            id: None,
            name: "update_monthly_to_never".to_string(),
            target: Decimal::new(1000, 0),
            target_date: Utc.with_ymd_and_hms(2025, 12, 31, 0, 0, 0).unwrap(),
            user_id,
            accumulated_amount: Decimal::ZERO,
            recurrence: Recurrence::Monthly,
            start_date: None,
        };
        let created = goal.create(&client).await.unwrap();
        goal.id = created.id;

        let request = Request::builder()
            .method(Method::POST)
            .uri(format!("/goals/{}", goal.id.unwrap()))
            .header("content-type", "application/x-www-form-urlencoded")
            .body(
                "name=Updated%20Goal&target=2000.0&target_date=2024-12-31&recurrence=never"
                    .to_string(),
            )
            .unwrap();

        // Create a test app and call the action
        let app = axum::Router::new()
            .route(
                "/goals/{id}",
                axum::routing::post(crate::authenticated::goals::update::action),
            )
            .with_state(shared_state.clone())
            .layer(user_extension)
            .layer(context_extension);

        let response = app.oneshot(request).await.unwrap();

        // Assert the response
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers().get("location").unwrap(), "/goals");

        let goal = client
            .query_one(
                "SELECT * FROM goals WHERE user_id = $1 LIMIT 1",
                &[&user_id],
            )
            .await
            .unwrap();

        assert_eq!(goal.get::<_, String>("name"), "Updated Goal");
        let start_date_str = goal
            .get::<_, chrono::DateTime<chrono::Utc>>("start_date")
            .to_rfc3339();
        assert!(chrono::DateTime::parse_from_rfc3339(&start_date_str).is_ok());
    }
    use crate::{
        models::goal::{Goal, Recurrence},
        test_utils::state_for_tests,
    };
    use axum::http::{Method, Request, StatusCode};
    use chrono::Utc;
    use rust_decimal::Decimal;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_update_goal() {
        let (shared_state, user_extension, context_extension) = state_for_tests().await.unwrap();
        let user_id = user_extension.0.id;

        let goal = Goal {
            id: None,
            accumulated_amount: Decimal::new(100, 0),
            user_id,
            name: "Test Goal".to_string(),
            target: Decimal::new(1000, 0),
            target_date: Utc::now(),
            recurrence: Recurrence::Weekly,
            start_date: None,
        };
        let client = shared_state.pool.get_client().await.unwrap();

        let mut goal = goal.create(&client).await.unwrap();

        goal.accumulated_amount = Decimal::new(100, 0);
        let goal = goal.update(&client).await.unwrap();

        let request = Request::builder()
            .method(Method::POST)
            .uri(format!("/goals/{}", goal.id.unwrap()))
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
            .with_state(shared_state.clone())
            .layer(user_extension)
            .layer(context_extension);

        let response = app.oneshot(request).await.unwrap();
        let client = shared_state.pool.get_client().await.unwrap();

        // Assert the response
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers().get("location").unwrap(), "/goals");

        let goal = Goal::get_one(&client, goal.id.unwrap(), user_id)
            .await
            .unwrap();

        assert_eq!(goal.name, "Updated Goal");
        assert_eq!(goal.target, Decimal::new(2000, 0));
        assert_eq!(goal.accumulated_amount, Decimal::ZERO);
    }
}
