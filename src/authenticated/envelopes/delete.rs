use crate::{
    SharedState, authenticated::UserExtension, errors::AppResponse, models::envelope::Envelope,
};
use axum::{
    Extension,
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse},
};
use tera::Context;

pub async fn modal(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
    Path(id): Path<i32>,
) -> AppResponse {
    let envelope = Envelope::get_one(&shared_state.client, id, user.id).await?;

    let tera = shared_state.tera.clone();
    let mut context = Context::new();
    context.insert("envelope", &envelope);
    let content = tera.render("envelopes/delete/confirm.html", &context)?;

    Ok(Html::from(content).into_response())
}

pub async fn action(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
    Path(id): Path<i32>,
) -> AppResponse {
    let envelope = Envelope::get_one(&shared_state.client, id, user.id).await?;

    envelope.delete(&shared_state.client).await?;

    let tera = shared_state.tera.clone();
    let mut context = Context::new();
    context.insert("envelope", &envelope);
    let content = tera.render("envelopes/delete.html", &context)?;

    Ok((
        StatusCode::OK,
        [("content-type", "text/vnd.turbo-stream.html")],
        Html::from(content),
    )
        .into_response())
}
