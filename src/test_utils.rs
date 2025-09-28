use crate::{
    HandlebarsContext,
    authenticated::UserExtension,
    errors::AppError,
    models::user::Preferences,
    utilities::handlebars::{DigestAssetHandlebarsHelper, walk_directory},
};
#[cfg(test)]
use crate::{SharedState, database_pool, models::user::User};
use anyhow::{Result, anyhow};
use axum::Extension;
use axum_extra::extract::cookie::Key;
use chrono::Utc;
use handlebars::Handlebars;
use postgres_types::Json;
use rust_database_common::GenericClient;

pub async fn state_for_tests() -> Result<(
    SharedState,
    Extension<UserExtension>,
    Extension<HandlebarsContext>,
)> {
    let pool = database_pool(Some(
        "postgres://simple_budget@localhost:5432/simple_budget_test",
    ))
    .await?;

    let client = pool.get_client().await?;

    let user_extension = user_extension_for_tests(&client).await.unwrap();
    let mut handlebars = Handlebars::new();
    handlebars.set_dev_mode(true);
    handlebars.register_helper(
        "digest_asset",
        Box::new(DigestAssetHandlebarsHelper {
            key: Utc::now().timestamp_millis().to_string(),
        }),
    );

    for template in walk_directory("./templates").unwrap() {
        let name = template
            .to_str()
            .unwrap()
            .replace("./templates/", "")
            .replace(".hbs", "");
        handlebars
            .register_template_file(&name, template.to_str().unwrap())
            .unwrap();
    }
    let pool = database_pool(Some(
        "postgres://simple_budget@localhost:5432/simple_budget_test",
    ))
    .await?;
    let shared_state = SharedState {
        key: Key::generate(),
        pool,
        handlebars,
    };

    Ok((
        shared_state,
        user_extension,
        Extension(HandlebarsContext::new()),
    ))
}

pub async fn user_for_tests(
    client: &impl GenericClient,
    preferences: Option<Preferences>,
) -> Result<User, AppError> {
    let user = User::create(
        client,
        uuid::Uuid::new_v4().to_string(),
        uuid::Uuid::new_v4().to_string(),
    )
    .await?;

    let preferences = preferences
        .or(Some(Preferences::default()))
        .ok_or(anyhow!("could not create preferences"))?;

    let mut user = user.clone();

    user.preferences = Some(Json(preferences));

    let user = user.update(client).await?;
    Ok(user)
}

async fn user_extension_for_tests(
    client: &impl GenericClient,
) -> Result<Extension<UserExtension>, AppError> {
    let user = user_for_tests(client, None).await?;

    Ok(Extension(UserExtension {
        id: user.id,
        csrf: "test".to_owned(),
    }))
}
