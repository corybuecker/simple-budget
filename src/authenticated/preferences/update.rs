use std::{collections::HashMap, str::FromStr};

use super::PreferencesForm;
use crate::{
    authenticated::{dashboard::generate_dashboard_context_for, UserExtension},
    errors::FormError,
    models::{goal::Goal, user::User},
    SharedState,
};
use axum::{
    extract::State,
    response::{Html, IntoResponse, Response},
    Extension, Form,
};
use bson::oid::ObjectId;
use chrono::Utc;
use mongodb::bson::doc;
use tera::Context;

pub async fn action(
    shared_state: State<SharedState>,
    user: Extension<UserExtension>,
    form: Form<PreferencesForm>,
) -> Result<Response, FormError> {
    let mut user = User::get_by_id(&shared_state.mongo, &user.id)
        .await
        .unwrap();

    if let Some(string) = &form.timezone {
        if string.is_empty() {
            user.preferences.timezone = None
        } else {
            user.preferences.timezone = Some(string.clone())
        }
    }

    if let Some(goal_header) = &form.goal_header {
        user.preferences.goal_header = Some(goal_header.to_owned());
    }

    match form.forecast_offset {
        None => {}
        Some(forecast_offset) => {
            if forecast_offset + 1 > 3 {
                user.preferences.forecast_offset = Some(1)
            } else {
                user.preferences.forecast_offset = Some(forecast_offset + 1)
            }
        }
    };

    let collection = shared_state
        .mongo
        .default_database()
        .unwrap()
        .collection::<User>("users");

    let _ = collection
        .update_one(doc! {"_id": ObjectId::from_str(&user._id).unwrap()}, doc! {
            "$set":doc! {
            "preferences": doc! {"timezone": &user.preferences.timezone, 
            "forecast_offset": &user.preferences.forecast_offset, "goal_header": &user.preferences.goal_header}}})
        .await?;

    let tera = &shared_state.tera;
    let mut goals_context = Context::new();
    let goal_header = &user.preferences.goal_header;
    let mut accumulations: HashMap<String, f64> = HashMap::new();
    let mut days_remainings: HashMap<String, i64> = HashMap::new();
    let goals = Goal::get_by_user_id(&shared_state.mongo, &user._id)
        .await
        .unwrap();

    goals_context.insert("goal_header", goal_header);

    for goal in &goals {
        accumulations.insert(goal._id.clone(), goal.accumulated());
        days_remainings.insert(goal._id.clone(), (goal.target_date - Utc::now()).num_days());
    }

    goals_context.insert("goals", &goals);
    goals_context.insert("accumulations", &accumulations);
    goals_context.insert("days_remainings", &days_remainings);
    goals_context.insert("goals", &goals);

    let goals_html = tera.render("goals/_table.html", &goals_context)?;

    let dashboard_context = generate_dashboard_context_for(&user, &shared_state.mongo).await;

    let dashboard_content = shared_state
        .tera
        .render("_dashboard.html", &dashboard_context)
        .unwrap();

    let mut context = Context::new();
    context.insert("goals_update", &goals_html);
    context.insert("dashboard_update", &dashboard_content);
    let html = tera.render("preferences/update.html", &context)?;

    Ok((
        [("content-type", "text/vnd.turbo-stream.html")],
        Html::from(html),
    )
        .into_response())
}
