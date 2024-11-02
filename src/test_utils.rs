#[cfg(test)]
use crate::extract_id;
use crate::{authenticated::UserExtension, digest_asset, mongo_client, Broker, SharedState};
use axum::Extension;
use axum_extra::extract::cookie::Key;
use tokio::sync::{mpsc, watch};

pub async fn state_for_tests() -> SharedState {
    let client = mongo_client().await.unwrap();
    let (sender, _rx) = mpsc::channel(1);
    let mut tera = tera::Tera::new("templates/**/*.html").unwrap();

    tera.register_function("digest_asset", digest_asset());
    tera.register_filter("oid", extract_id());

    SharedState {
        mongo: client,
        key: Key::generate(),
        broker: Broker { sender },
        tera,
    }
}

pub fn user_for_tests(user_id: &str) -> Extension<UserExtension> {
    let (tx, rx) = watch::channel("".to_owned());

    Extension(UserExtension {
        id: user_id.to_owned(),
        csrf: "test".to_owned(),
        channel_sender: tx,
        channel_receiver: rx,
    })
}
