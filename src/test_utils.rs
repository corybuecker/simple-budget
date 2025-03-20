#[cfg(test)]
use crate::{
    Broker, SharedState, authenticated::UserExtension, database_pool, digest_asset,
    models::user::User,
};
use crate::{errors::AppError, models::user::Preferences};
use anyhow::{Result, anyhow};
use axum::Extension;
use axum_extra::extract::cookie::Key;
use deadpool_postgres::Object;
use postgres_types::Json;
use tokio::sync::{mpsc, watch};
use tokio_postgres::{Client, GenericClient};

pub async fn state_for_tests() -> Result<(SharedState, Extension<UserExtension>)> {
    let pool = database_pool(Some(
        "postgres://simple_budget@localhost:5432/simple_budget_test",
    ))
    .await?;

    let (sender, _rx) = mpsc::channel(1);
    let mut tera = tera::Tera::new("templates/**/*.html").unwrap();

    tera.register_function("digest_asset", digest_asset());

    let user_extension = user_extension_for_tests(pool.get().await?.client())
        .await
        .unwrap();

    let shared_state = SharedState {
        key: Key::generate(),
        broker: Broker { sender },
        tera,
        pool,
    };

    Ok((shared_state, user_extension))
}

pub async fn client_for_tests() -> Result<Object> {
    let pool = database_pool(Some(
        "postgres://simple_budget@localhost:5432/simple_budget_test",
    ))
    .await?;
    let manager = pool.get().await?;

    Ok(manager)
}

pub async fn user_for_tests(
    client: &Client,
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

async fn user_extension_for_tests(client: &Client) -> Result<Extension<UserExtension>, AppError> {
    let (tx, rx) = watch::channel("".to_owned());

    let user = user_for_tests(client, None).await?;

    Ok(Extension(UserExtension {
        id: user.id,
        csrf: "test".to_owned(),
        channel_sender: tx,
        channel_receiver: rx,
    }))
}
