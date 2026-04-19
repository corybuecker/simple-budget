use anyhow::Result;
use base64::{Engine, engine::general_purpose};
use rust_database_common::DatabasePool;
use std::env;

pub async fn database_pool(database_url: Option<&str>) -> Result<DatabasePool> {
    let database_url = match database_url {
        Some(url) => url,
        None => &env::var("DATABASE_URL")?,
    };

    let secure = env::var("DATABASE_CA_CERT").is_ok_and(|s| !s.is_empty());

    match secure {
        true => {
            let ca_certificate = env::var("DATABASE_CA_CERT")?;
            let ca_certificate = general_purpose::STANDARD.decode(&ca_certificate)?;
            let ca_certificate = String::from_utf8(ca_certificate)?;

            let mut pool =
                DatabasePool::new(database_url.to_string()).with_required_ssl_mode(ca_certificate);
            pool.connect().await?;
            Ok(pool)
        }
        false => {
            let mut pool = DatabasePool::new(database_url.to_string());
            pool.connect().await?;
            Ok(pool)
        }
    }
}
