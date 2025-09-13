use super::{GoalForm, schema};
use crate::{
    HandlebarsContext, SharedState,
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
use chrono::{NaiveDateTime, NaiveTime, Utc};
use handlebars::to_json;
use rust_decimal::{Decimal, prelude::FromPrimitive};
use std::str::FromStr;
use tokio_postgres::GenericClient;

pub async fn action(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
    headers: HeaderMap,
    Extension(context): Extension<HandlebarsContext>,
    Form(form): Form<GoalForm>,
) -> AppResponse {
    let json = serde_json::to_value(&form)?;
    let valid = jsonschema::validate(&schema(), &json);
    let response_format = responses::get_response_format(&headers)?;

    match valid {
        Ok(_) => {}
        Err(validation_errors) => {
            let mut context = context.clone();

            context.insert("errors".to_string(), to_json(validation_errors.to_string()));
            context.insert("name".to_string(), to_json(&form.name));
            context.insert("target".to_string(), to_json(form.target));
            context.insert("target_date".to_string(), to_json(form.target_date));
            context.insert("recurrence".to_string(), to_json(&form.recurrence));

            match response_format {
                responses::ResponseFormat::Html => {
                    context.insert("partial".to_string(), to_json("goals/new"));
                    return Ok(responses::generate_response(
                        &responses::ResponseFormat::Html,
                        shared_state.handlebars.render("layout", &context)?,
                        StatusCode::BAD_REQUEST,
                    ));
                }
                responses::ResponseFormat::Turbo => {
                    return Ok(responses::generate_response(
                        &response_format,
                        shared_state
                            .handlebars
                            .render("goals/_form.turbo", &context)?,
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

    let recurrence = Recurrence::from_str(&form.recurrence).map_err(|e| anyhow!("{:#?}", e))?;
    let start_date = match recurrence {
        Recurrence::Never => Some(Utc::now()),
        _ => None,
    };

    let goal = Goal {
        id: None,
        name: form.name.to_owned(),
        target: Decimal::from_f64(form.target.to_owned())
            .ok_or_else(|| anyhow!("could not parse decimal"))?,
        target_date: NaiveDateTime::new(form.target_date, NaiveTime::MIN).and_utc(),
        user_id: user.id,
        accumulated_amount: Decimal::ZERO,
        recurrence,
        start_date,
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
        let (shared_state, user_extension, context_extension) = state_for_tests().await.unwrap();
        let client = shared_state.pool.get().await.unwrap();
        let client = client.client();
        let user_id = user_extension.0.id;

        let app = Router::new()
            .route("/goals/create", post(action))
            .with_state(shared_state.clone())
            .layer(user_extension)
            .layer(context_extension);

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
        let (shared_state, user_extension, context_extension) = state_for_tests().await.unwrap();

        let app = Router::new()
            .route("/goals/create", post(action))
            .with_state(shared_state)
            .layer(user_extension)
            .layer(context_extension);

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
        assert!(body_str.contains("124"))
    }

    #[tokio::test]
    async fn test_create_goal_turbo_stream() {
        let (shared_state, user_extension, context_extension) = state_for_tests().await.unwrap();
        let app = Router::new()
            .route("/goals/create", post(action))
            .with_state(shared_state.clone())
            .layer(user_extension)
            .layer(context_extension);

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

    #[tokio::test]
    async fn test_create_goal_with_explicit_start_date_never_recurrence() {
        let (shared_state, user_extension, context_extension) = state_for_tests().await.unwrap();
        let client = shared_state.pool.get().await.unwrap();
        let client = client.client();
        let user_id = user_extension.0.id;

        let app = Router::new()
            .route("/goals/create", post(action))
            .with_state(shared_state.clone())
            .layer(user_extension)
            .layer(context_extension);

        let target_date = Utc::now().add(Duration::days(7));

        let form_data = format!(
            "name=test_create_goal_success&target=124&target_date={}&recurrence=never",
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

        assert_eq!(goal.get::<_, String>("name"), "test_create_goal_success");
        let start_date_str = goal
            .get::<_, chrono::DateTime<chrono::Utc>>("start_date")
            .to_rfc3339();
        assert!(chrono::DateTime::parse_from_rfc3339(&start_date_str).is_ok());
    }
}
