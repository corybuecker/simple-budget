use crate::{
    SharedState, authenticated::UserExtension, errors::AppResponse, models::envelope::Envelope,
};
use axum::{
    Extension,
    extract::{Path, State},
    response::{Html, IntoResponse},
};
use tera::Context;

pub async fn page(
    shared_state: State<SharedState>,
    Path(id): Path<i32>,
    user: Extension<UserExtension>,
    Extension(mut context): Extension<Context>,
) -> AppResponse {
    let envelope = Envelope::get_one(&shared_state.client, id, user.id).await?;

    context.insert("id", &envelope.id);
    context.insert("name", &envelope.name);
    context.insert("amount", &envelope.amount);

    let content = shared_state.tera.render("envelopes/edit.html", &context)?;

    Ok(Html::from(content).into_response())
}
