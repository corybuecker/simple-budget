use std::collections::HashMap;

use crate::{
    authenticated::UserExtension,
    errors::FormError,
    models::{
        goal::Goal,
        user::{GoalHeader, User},
    },
    SharedState,
};
use axum::{
    extract::State,
    response::{Html, IntoResponse, Response},
    Extension, Form,
};
use bson::{doc, oid::ObjectId};
use chrono::Utc;
use serde::Deserialize;
use tera::Context;

#[derive(Debug, Deserialize)]
pub struct HeaderForm {
    goal_header: GoalHeader,
}

pub async fn action(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
    form: Form<HeaderForm>,
) -> Result<Response, FormError> {
    let user = User::get_by_id(&shared_state.mongo, &user.id)
        .await
        .unwrap();

    shared_state
        .mongo
        .default_database()
        .unwrap()
        .collection::<User>("users")
        .update_one(
            doc! {"_id": ObjectId::parse_str(&user._id).unwrap()},
            doc! {"$set": doc! {"preferences.goal_header": form.goal_header.clone()}},
        )
        .await?;

    let goals = Goal::get_by_user_id(&shared_state.mongo, &user._id)
        .await
        .unwrap();

    let goal_header = &form.goal_header;
    let mut context = Context::new();
    context.insert("goal_header", goal_header);

    let mut accumulations: HashMap<String, f64> = HashMap::new();
    let mut days_remainings: HashMap<String, i64> = HashMap::new();

    for goal in &goals {
        accumulations.insert(goal._id.clone(), goal.accumulated());
        days_remainings.insert(goal._id.clone(), (goal.target_date - Utc::now()).num_days());
    }

    context.insert("goals", &goals);
    context.insert("accumulations", &accumulations);
    context.insert("days_remainings", &days_remainings);

    let tera = &shared_state.tera;

    let html = tera.render("goals/index.html", &context)?;

    Ok(Html::from(html).into_response())
}
