use crate::{
    authenticated::UserExtension, errors::FormError, models::envelope::Envelope, SharedState,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Extension,
};
use bson::{doc, oid::ObjectId};
use std::str::FromStr;
use tera::Context;

pub async fn page(
    shared_state: State<SharedState>,
    Path(id): Path<String>,
    user: Extension<UserExtension>,
) -> Result<Response, FormError> {
    let envelopes: mongodb::Collection<Envelope> = shared_state
        .mongo
        .default_database()
        .unwrap()
        .collection("envelopes");

    let envelope = envelopes
        .find_one(            doc! {"_id": ObjectId::from_str(&id).unwrap(), "user_id": ObjectId::from_str(&user.id).unwrap()} ).await?;

    let Some(envelope) = envelope else {
        return Err(FormError {
            message: "could not find envelope".to_string(),
            status_code: Some(StatusCode::NOT_FOUND),
        });
    };

    let mut context = Context::new();
    context.insert("csrf", &user.csrf);

    context.insert("id", &envelope._id);
    context.insert("name", &envelope.name);
    context.insert("amount", &envelope.amount);

    let content = shared_state.tera.render("envelopes/edit.html", &context)?;

    Ok(Html::from(content).into_response())
}
