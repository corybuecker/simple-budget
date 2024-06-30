use crate::{authenticated::UserExtension, SharedState};
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Extension,
};
use bson::{doc, oid::ObjectId};
use mongodb::Collection;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tera::Context;

#[derive(Serialize, Deserialize)]
struct Envelope {
    name: String,
    amount: f64,
    _id: ObjectId,
}

#[derive(Serialize)]
struct EnvelopeRecord {
    name: String,
    amount: f64,
    id: String,
}

pub async fn page(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
) -> Result<Response, StatusCode> {
    log::debug!("{:?}", user);
    let Ok(user_id) = ObjectId::from_str(&user.id) else {
        return Err(StatusCode::FORBIDDEN);
    };

    let collection: Collection<Envelope> = shared_state
        .mongo
        .database("simple_budget")
        .collection("envelopes");

    let mut context = Context::new();
    let mut envelopes: Vec<EnvelopeRecord> = Vec::new();

    match collection.find(doc! {"user_id": &user_id}, None).await {
        Ok(mut cursor) => {
            while cursor.advance().await.unwrap() {
                match cursor.deserialize_current() {
                    Ok(envelope) => {
                        envelopes.push(EnvelopeRecord {
                            name: envelope.name,
                            amount: envelope.amount,
                            id: envelope._id.to_string(),
                        });
                    }
                    Err(e) => {
                        log::error!("{}", e);
                    }
                }
            }
            context.insert("envelopes", &envelopes);
        }
        Err(e) => {
            log::error!("{}", e);
            context.insert("envelopes", &envelopes);
        }
    }

    let Ok(content) = shared_state.tera.render("envelopes/index.html", &context) else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    Ok(Html::from(content).into_response())
}
