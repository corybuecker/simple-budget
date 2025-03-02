use crate::{SharedState, authenticated::UserExtension, errors::FormError, models::goal::Goal};
use axum::{
    Extension,
    extract::{Path, State},
    response::{Html, IntoResponse, Response},
};
use tera::Context;

pub async fn page(
    shared_state: State<SharedState>,
    Path(id): Path<i32>,
    Extension(user): Extension<UserExtension>,
    Extension(mut context): Extension<Context>,
) -> Result<Response, FormError> {
    let goal = Goal::get_one(&shared_state.client, id, user.id).await?;
    context.insert("id", &goal.id);
    context.insert("name", &goal.name);
    context.insert("target", &goal.target);
    context.insert("target_date", &goal.target_date.date_naive());
    context.insert("recurrence", &goal.recurrence);

    let content = shared_state.tera.render("goals/edit.html", &context)?;

    Ok(Html::from(content).into_response())
}
