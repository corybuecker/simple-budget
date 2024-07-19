use crate::{authenticated::UserExtension, SharedState};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    Extension, Form,
};
use bson::{doc, oid::ObjectId};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tera::Context;
use validator::Validate;

#[derive(Debug, Validate, Deserialize)]
pub struct Envelope {
    #[validate(length(min = 5))]
    name: String,
    #[validate(range(min = 0.0))]
    amount: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EnvelopeRecord {
    name: String,
    amount: f64,
    user_id: ObjectId,
}

#[derive(Debug)]
pub struct Error {
    message: String,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        return (StatusCode::BAD_REQUEST, format!("{:#?}", self)).into_response();
    }
}

impl From<bson::oid::Error> for Error {
    fn from(value: bson::oid::Error) -> Self {
        Error {
            message: value.to_string(),
        }
    }
}

impl From<tera::Error> for Error {
    fn from(value: tera::Error) -> Self {
        Error {
            message: value.to_string(),
        }
    }
}
impl From<mongodb::error::Error> for Error {
    fn from(value: mongodb::error::Error) -> Self {
        Error {
            message: value.to_string(),
        }
    }
}

pub async fn page(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
    Path(id): Path<String>,
    headers: HeaderMap,
    form: Form<Envelope>,
) -> Result<Response, Error> {
    log::debug!("{:?}", user);
    log::debug!("{:?}", form);

    let mut turbo = false;
    let accept = headers.get("Accept");
    match accept {
        Some(accept) => {
            if accept.to_str().unwrap().contains("turbo") {
                turbo = true;
            }
        }
        _ => {}
    }

    match form.validate() {
        Ok(_) => {}
        Err(validation_errors) => {
            let mut context = Context::new();

            context.insert("errors", &validation_errors.to_string());
            context.insert("id", &id);
            context.insert("name", &form.name);
            context.insert("amount", &form.amount);

            let Ok(content) = shared_state.tera.render(
                if turbo {
                    "envelopes/edit.turbo.html"
                } else {
                    "envelopes/edit.html"
                },
                &context,
            ) else {
                return Err(Error {
                    message: "cannot render".to_owned(),
                });
            };

            let content = shared_state.tera.render("envelopes/edit.html", &context)?;
            return Ok((StatusCode::BAD_REQUEST, Html::from(content)).into_response());
        }
    }

    let envelopes: mongodb::Collection<EnvelopeRecord> = shared_state
        .mongo
        .database("simple_budget")
        .collection("envelopes");

    let filter = doc! {"_id": ObjectId::from_str(&id).unwrap(), "user_id": ObjectId::from_str(&user.id).unwrap()};
    log::debug!("{:?}", filter);

    let envelope = envelopes.find_one(filter.clone()).await?;

    let Some(mut envelope) = envelope else {
        return Err(Error {
            message: "could not update envelope".to_string(),
        });
    };

    envelope.name = form.name.clone();
    envelope.amount = form.amount;
    let _ = envelopes.replace_one(filter, envelope).await;

    Ok(Redirect::to("/envelopes").into_response())
}
