use crate::{database_client, models::user::Session};
use anyhow::{Context, Result};
use chrono::Utc;
use tracing::{debug, info};

pub async fn clear_sessions() -> Result<()> {
    info!("clearing old sessions at {}", Utc::now());
    let client = database_client(None).await?;
    let count = Session::delete_expired(&client)
        .await
        .context("could not delete sessions")?;
    debug!("deleted {} sessions", count);
    Ok(())
}
