use std::env;

use crate::{
    LayoutCtx, Section, authenticated::UserExtension, errors::AppError,
    models::user::preferences::Preferences,
};

#[cfg(test)]
use crate::{SharedState, db::database_pool, models::user::User};
use anyhow::{Result, anyhow};
use axum::Extension;
use axum_extra::extract::cookie::Key;
use postgres_types::Json;
use rust_database_common::GenericClient;

pub async fn state_for_tests()
-> Result<(SharedState, Extension<UserExtension>, Extension<LayoutCtx>)> {
    let pool = database_pool(Some(&env::var("TEST_DATABASE_URL")?)).await?;

    let user_extension = user_extension_for_tests(&pool.get_client().await?)
        .await
        .unwrap();

    let shared_state = SharedState {
        key: Key::generate(),
        pool,
    };

    let layout = LayoutCtx {
        section: Section::Reports,
        csrf_token: Some(user_extension.csrf.clone()),
        nonce: "test".to_owned(),
    };

    Ok((shared_state, user_extension, Extension(layout)))
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
