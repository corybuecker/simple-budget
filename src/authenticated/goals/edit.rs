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

pub async fn action(
    shared_state: State<SharedState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
    user: Extension<UserExtension>,
    Extension(context): Extension<HandlebarsContext>,
) -> AppResponse {
    let client = shared_state.pool.get_client().await?;
    let goal = Goal::get_one(&client, id, user.id).await?;
    let mut context = context.clone();
    let response_format = get_response_format(&headers)?;
    context.insert("id".to_string(), to_json(goal.id));
    context.insert("name".to_string(), to_json(&goal.name));
    context.insert("target".to_string(), to_json(goal.target));
    context.insert(
        "target_date".to_string(),
        to_json(goal.target_date.format("%Y-%m-%d").to_string()),
    );
    context.insert(
        "recurrence".to_string(),
        to_json(format!("{:?}", goal.recurrence).to_lowercase()),
    );
    match response_format {
        ResponseFormat::Html => {
            context.insert("partial".to_string(), to_json("goals/edit"));

            Ok(generate_response(
                &response_format,
                shared_state.handlebars.render("layout", &context)?,
                StatusCode::OK,
            ))
        }
        ResponseFormat::Turbo => Ok(StatusCode::BAD_REQUEST.into_response()),
        ResponseFormat::Json => Ok(generate_response(
            &response_format,
            Json(goal),
            StatusCode::OK,
        )),
    }
}
