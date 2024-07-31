use crate::{authenticated::UserExtension, models::goal::Goal, SharedState};
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Extension,
};
use bson::{doc, oid::ObjectId};
use tera::Context;

pub async fn page(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
) -> Result<Response, StatusCode> {
    log::debug!("{:?}", user);

    let Ok(user_id) = ObjectId::parse_str(&user.id) else {
        return Err(StatusCode::FORBIDDEN);
    };

    let collection = shared_state
        .mongo
        .database("simple_budget")
        .collection::<Goal>("goals");

    let mut context = Context::new();
    let mut goals: Vec<Goal> = Vec::new();
    context.insert("csrf", &user.csrf);

    match collection.find(doc! {"user_id": &user_id}).await {
        Ok(mut cursor) => {
            while cursor.advance().await.unwrap() {
                match cursor.deserialize_current() {
                    Ok(goal) => {
                        goals.push(goal);
                    }
                    Err(e) => {
                        log::error!("{}", e);
                    }
                }
            }
            context.insert("goals", &goals);
        }
        Err(e) => {
            log::error!("{}", e);
            context.insert("goals", &goals);
        }
    }

    let content = shared_state
        .tera
        .render("goals/index.html", &context)
        .unwrap();

    Ok(Html::from(content).into_response())
}
