use crate::{
    authenticated::{FormError, UserExtension},
    SharedState,
};
use axum::{
    extract::State,
    response::{Html, IntoResponse, Response},
    Extension,
};
use bson::serde_helpers::hex_string_as_object_id;
use bson::{doc, oid::ObjectId};
use mongodb::Collection;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tera::Context;

#[derive(Serialize, Deserialize)]
struct Envelope {
    name: String,
    amount: f64,
    #[serde(with = "hex_string_as_object_id")]
    _id: String,
}

pub async fn page(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
) -> Result<Response, FormError> {
    let user_id = ObjectId::from_str(&user.id)?;

    let collection: Collection<Envelope> = shared_state
        .mongo
        .database("simple_budget")
        .collection("envelopes");

    let mut context = Context::new();
    context.insert("csrf", &user.csrf);
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
