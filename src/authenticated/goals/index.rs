use crate::{authenticated::UserExtension, errors::FormError, models::goal::Goal, SharedState};
use axum::{
    extract::State,
    response::{Html, IntoResponse, Response},
    Extension,
};
use bson::{doc, oid::ObjectId};
use std::collections::HashMap;
use tera::Context;

pub async fn page(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
) -> Result<Response, FormError> {
    let user_id = ObjectId::parse_str(&user.id)?;

    let collection = shared_state
        .mongo
        .default_database()
        .unwrap()
        .collection::<Goal>("goals");

    let mut context = Context::new();
    let mut goals: Vec<Goal> = Vec::new();

    let mut accumulations: HashMap<String, f64> = HashMap::new();

    context.insert("csrf", &user.csrf);

    match collection.find(doc! {"user_id": &user_id}).await {
        Ok(mut cursor) => {
            while cursor.advance().await.unwrap() {
                match cursor.deserialize_current() {
                    Ok(goal) => {
                        accumulations.insert(goal._id.clone(), goal.accumulated());
                        goals.push(goal);
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

    context.insert("goals", &goals);
    context.insert("accumulations", &accumulations);

    let content = shared_state.tera.render("goals/index.html", &context)?;

    Ok(Html::from(content).into_response())
}
