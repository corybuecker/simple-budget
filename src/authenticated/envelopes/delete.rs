use crate::{
    authenticated::UserExtension, errors::FormError, models::envelope::Envelope, SharedState,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Extension,
};
use mongodb::bson::{doc, oid::ObjectId};
use std::str::FromStr;
use tera::Context;

pub async fn modal(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
    Path(id): Path<String>,
) -> Result<Response, FormError> {
    let envelopes: mongodb::Collection<Envelope> = shared_state
        .mongo
        .default_database()
        .unwrap()
        .collection("envelopes");

    let filter = doc! {"_id": ObjectId::from_str(&id).unwrap(), "user_id": ObjectId::from_str(&user.id).unwrap()};

    let envelope = envelopes.find_one(filter.clone()).await?;

    let Some(_) = envelope else {
        return Err(FormError {
            message: "could not find envelope".to_string(),
            status_code: Some(StatusCode::NOT_FOUND),
        });
    };
    let tera = shared_state.tera.clone();
    let mut context = Context::new();
    context.insert("envelope", &envelope);
    let content = tera.render("envelopes/delete/confirm.html", &context)?;

    Ok(Html::from(content).into_response())
}

pub async fn action(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
    Path(id): Path<String>,
) -> Result<Response, FormError> {
    let envelopes: mongodb::Collection<Envelope> = shared_state
        .mongo
        .default_database()
        .unwrap()
        .collection("envelopes");

    let filter = doc! {"_id": ObjectId::from_str(&id).unwrap(), "user_id": ObjectId::from_str(&user.id).unwrap()};
    let envelope = envelopes.find_one(filter.clone()).await?;
    let Some(_) = envelope else {
        return Ok((StatusCode::NOT_FOUND, Html::from("Not Found")).into_response());
    };
    let _ = envelopes.delete_one(filter).await?;

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
