use super::PreferencesForm;
use crate::{authenticated::UserExtension, errors::FormError, models::user::User, SharedState};
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    Extension, Form,
};
use mongodb::{
    bson::{doc, oid::ObjectId},
    Collection,
};
use std::str::FromStr;
use tera::Context;
use tracing::debug;
use validator::Validate;

pub async fn action(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
    headers: HeaderMap,
    form: Form<PreferencesForm>,
) -> Result<Response, FormError> {
    let mut turbo = false;
    let accept = headers.get("Accept");
    if let Some(accept) = accept {
        if accept.to_str().unwrap().contains("turbo") {
            turbo = true;
        }
    }
    match form.validate() {
        Ok(_) => {}
        Err(validation_errors) => {
            debug!("{:#?}", validation_errors);
            let mut context = Context::new();

            context.insert("errors", &validation_errors.to_string());
            context.insert("timezone", &form.timezone);
            let content = shared_state.tera.render(
                if turbo {
                    "preferences/form.turbo.html"
                } else {
                    "preferences/edit.html"
                },
                &context,
            )?;

            if turbo {
                return Ok((
                    StatusCode::BAD_REQUEST,
                    [("content-type", "text/vnd.turbo-stream.html")],
                    Html::from(content),
                )
                    .into_response());
            } else {
                return Ok((StatusCode::BAD_REQUEST, Html::from(content)).into_response());
            }
        }
    }

    let database = &shared_state.mongo.default_database().unwrap();
    let collection: Collection<User> = database.collection("users");
    let user = collection
        .find_one(doc! {"_id": ObjectId::from_str(&user.id).unwrap()})
        .await?;

    let Some(mut user) = user else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    match &form.timezone {
        Some(string) => {
            if string.is_empty() {
                user.preferences.timezone = None
            } else {
                user.preferences.timezone = Some(string.clone())
            }
        }
        None => user.preferences.timezone = None,
    }

    collection
        .update_one(
            doc! {"_id": ObjectId::from_str(&user._id).unwrap()},
            doc! {"$set": doc! {"preferences.timezone": user.preferences.timezone}},
        )
        .await?;

    Ok(Redirect::to("/preferences").into_response())
}
