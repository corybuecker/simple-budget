use crate::{
    SharedState,
    authenticated::UserExtension,
    errors::FormError,
    models::envelope::{self, Envelope},
};
use anyhow::Result;
use axum::{
    Extension,
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use tera::Context;

pub async fn page(
    shared_state: State<SharedState>,
    Path(id): Path<String>,
    user: Extension<UserExtension>,
    Extension(mut context): Extension<Context>,
) -> Result<Response> {
    let envelope = Envelope::get_by_user_id(
        &shared_state.client,
        user.id.parse::<i32>()?,
        id.parse::<i32>()?,
    )
    .await?;

    context.insert("id", &envelope.id);
    context.insert("name", &envelope.name);
    context.insert("amount", &envelope.amount);

    let content = shared_state.tera.render("envelopes/edit.html", &context)?;

    Ok(Html::from(content).into_response())
}
