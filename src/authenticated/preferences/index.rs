use crate::{
    SharedState,
    authenticated::UserExtension,
    errors::FormError,
    models::user::{Preferences, User},
};
use axum::{
    Extension,
    extract::State,
    response::{Html, IntoResponse, Response},
};
use postgres_types::Json;
use tera::{Context, Tera};
use tokio_postgres::Client;

pub async fn action(
    state: State<SharedState>,
    user: Extension<UserExtension>,
    Extension(mut context): Extension<Context>,
) -> Result<Response, FormError> {
    let client: &Client = &state.client;

    let user = User::get_by_id(client, user.id).await?;
    let preferences = user
        .preferences
        .unwrap_or(Json(Preferences::default()))
        .0;

    context.insert("timezone", &preferences.timezone);

    let tera: &Tera = &state.tera;
    let content = tera.render("preferences/index.html", &context)?;

    Ok(Html::from(content).into_response())
}
