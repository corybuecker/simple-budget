use crate::{
    authenticated::UserExtension, errors::FormError, models::envelope::Envelope, SharedState,
};
use axum::{
    extract::State,
    response::{Html, IntoResponse, Response},
    Extension,
};
use bson::{doc, oid::ObjectId};
use mongodb::Collection;
use std::str::FromStr;
use tera::Context;

pub async fn page(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
    Extension(mut context): Extension<Context>,
) -> Result<Response, FormError> {
    let user_id = ObjectId::from_str(&user.id)?;

    let collection: Collection<Envelope> = shared_state
        .mongo
        .default_database()
        .unwrap()
        .collection("envelopes");

    let mut envelopes: Vec<Envelope> = Vec::new();

    match collection.find(doc! {"user_id": &user_id}).await {
        Ok(mut cursor) => {
            while cursor.advance().await.unwrap() {
                match cursor.deserialize_current() {
                    Ok(envelope) => {
                        envelopes.push(envelope);
                    }
                    Err(e) => {
                        log::error!("{:#?}", e);
                    }
                }
            }
        }
        Err(e) => {
            log::error!("{:#?}", e);
        }
    }

    context.insert("envelopes", &envelopes);
    let content = shared_state.tera.render("envelopes/index.html", &context)?;

    Ok(Html::from(content).into_response())
}
