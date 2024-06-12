use crate::api::UserExtension;
use axum::Extension;

pub async fn index(user: Extension<UserExtension>) -> &'static str {
    log::debug!("{:?}", user);
    "Hello, World!"
}
