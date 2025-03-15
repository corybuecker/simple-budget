use crate::{
    SharedState, authenticated::UserExtension, errors::AppResponse, models::envelope::Envelope,
};
use axum::{
    Extension,
    extract::State,
    response::{Html, IntoResponse},
};
use tera::Context;

pub async fn page(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
    Extension(mut context): Extension<Context>,
) -> AppResponse {
    let envelopes = Envelope::get_all(&shared_state.client, user.id).await?;
    context.insert("envelopes", &envelopes);
    let content = shared_state.tera.render("envelopes/index.html", &context)?;

    Ok(Html::from(content).into_response())
}
