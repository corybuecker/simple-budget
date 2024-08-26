use super::EnvelopeForm;
use crate::{
    authenticated::UserExtension, errors::FormError, models::envelope::Envelope, SharedState,
};
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    Extension, Form,
};
use bson::oid::ObjectId;
use mongodb::Collection;
use std::str::FromStr;
use tera::Context;
use validator::Validate;

pub async fn page(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
    headers: HeaderMap,
    form: Form<EnvelopeForm>,
) -> Result<Response, FormError> {
    let mut context = Context::new();

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
            context.insert("errors", &validation_errors.to_string());
            context.insert("name", &form.name);
            context.insert("amount", &form.amount);

            let content = shared_state.tera.render(
                if turbo {
                    "envelopes/form.turbo.html"
                } else {
                    "envelopes/new.html"
                },
                &context,
            )?;

            if turbo {
                return Ok((
                    StatusCode::UNPROCESSABLE_ENTITY,
                    [("content-type", "text/vnd.turbo-stream.html")],
                    Html::from(content),
                )
                    .into_response());
            } else {
                return Ok((StatusCode::UNPROCESSABLE_ENTITY, Html::from(content)).into_response());
            }
        }
    }

    let goal_record = Envelope {
        _id: ObjectId::new().to_string(),
        name: form.name.to_owned(),
        amount: form.amount.to_owned(),
        user_id: ObjectId::from_str(&user.id).unwrap().to_string(),
    };
    let goals: Collection<Envelope> = shared_state
        .mongo
        .default_database()
        .unwrap()
        .collection("envelopes");

    let _ = goals.insert_one(goal_record).await?;

    Ok(Redirect::to("/envelopes").into_response())
}
