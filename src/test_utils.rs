#[cfg(test)]
use crate::database_client;
use crate::{Broker, SharedState, authenticated::UserExtension, digest_asset, models::user::User};
use anyhow::Result;
use axum::Extension;
use axum_extra::extract::cookie::Key;
use tokio::sync::{mpsc, watch};
use tokio_postgres::Client;

pub async fn state_for_tests() -> Result<(SharedState, Extension<UserExtension>)> {
    let client = database_client(Some(
        "postgres://postgres@localhost:5432/simple_budget_test",
    ))
    .await?;

    let (sender, _rx) = mpsc::channel(1);
    let mut tera = tera::Tera::new("templates/**/*.html").unwrap();

    tera.register_function("digest_asset", digest_asset());

    let user_extension = user_for_tests(&client).await?;

    let shared_state = SharedState {
        client: client.into(),
        key: Key::generate(),
        broker: Broker { sender },
        tera,
    };

    Ok((shared_state, user_extension))
}

async fn user_for_tests(client: &Client) -> Result<Extension<UserExtension>> {
    let (tx, rx) = watch::channel("".to_owned());

    let user = User::create(
        client,
        uuid::Uuid::new_v4().to_string(),
        uuid::Uuid::new_v4().to_string(),
    )
    .await?;

    Ok(Extension(UserExtension {
        id: user.id,
        csrf: "test".to_owned(),
        channel_sender: tx,
        channel_receiver: rx,
    }))
}
