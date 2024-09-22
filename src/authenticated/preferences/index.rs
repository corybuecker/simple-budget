use crate::{authenticated::UserExtension, errors::FormError, models::user::User, SharedState};
use axum::{
    extract::State,
    response::{Html, IntoResponse, Response},
    Extension,
};
use bson::{doc, oid::ObjectId};
use mongodb::Client;
use std::str::FromStr;
use tera::{Context, Tera};

pub async fn action(
    state: State<SharedState>,
    user: Extension<UserExtension>,
    Extension(mut context): Extension<Context>,
) -> Result<Response, FormError> {
    let client: &Client = &state.mongo;

    let user = client
        .default_database()
        .expect("could not connect")
        .collection::<User>("users")
        .find_one(doc! {"_id": ObjectId::from_str(&user.id)?})
        .await?
        .expect("could not find user");

    context.insert("timezone", &user.preferences.timezone);

    let tera: &Tera = &state.tera;
    let content = tera.render("preferences/index.html", &context)?;

    Ok(Html::from(content).into_response())
}
