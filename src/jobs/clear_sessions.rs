use crate::models::user::Session;
use anyhow::{Context, Result};
use chrono::Utc;
use rust_database_common::DatabasePool;
use tracing::{debug, info};

pub async fn clear_sessions(pool: &DatabasePool) -> Result<()> {
    info!("clearing old sessions at {}", Utc::now());
    let client = pool.get_client().await?;
    let count = Session::delete_expired(&client)
        .await
        .context("could not delete sessions")?;
    debug!("deleted {} sessions", count);
    Ok(())
}
