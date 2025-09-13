mod authenticated;
mod authentication;
mod errors;
mod jobs;
mod models;
mod utilities;

use crate::utilities::handlebars::{
    DigestAssetHandlebarsHelper, EqHandlebarsHelper, walk_directory,
};
use anyhow::Result;
use axum::{
    Router,
    extract::{FromRef, Request},
    http::{HeaderValue, StatusCode},
    middleware::{Next, from_fn},
    response::{IntoResponse, Response},
    routing::get,
};
use axum_extra::extract::cookie::Key;
use chrono::Utc;
use deadpool_postgres::{Config, Pool, RecyclingMethod, Runtime};
use errors::AppResponse;
use handlebars::Handlebars;
use jobs::{clear_sessions::clear_sessions, convert_goals::convert_goals};
use rust_web_common::telemetry::TelemetryBuilder;
use serde::Serialize;
use std::{collections::BTreeMap, env, time::Duration};
use tokio::{
    select,
    signal::unix::{SignalKind, signal},
    spawn,
    time::interval,
};
use tokio_postgres::{Client, NoTls};
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing::debug;
use utilities::dates::TimeProvider;

#[derive(Serialize, Clone)]
pub enum Section {
    Reports,
    Accounts,
    Envelopes,
    Goals,
    Preferences,
}

pub type HandlebarsContext = BTreeMap<String, serde_json::Value>;

#[derive(Clone, Debug)]
pub struct SharedState {
    key: Key,
    pool: Pool,
    handlebars: Handlebars<'static>,
}

impl FromRef<SharedState> for Key {
    fn from_ref(state: &SharedState) -> Self {
        state.key.clone()
    }
}

fn start_background_jobs() -> tokio::task::JoinHandle<()> {
    spawn(async {
        let mut interval = interval(Duration::from_millis(60000));
        let jobs_pool = database_pool(None).await.unwrap();
        let time = TimeProvider {};

        loop {
            interval.tick().await;

            let (_clear_sessions_result, _convert_goals_result) =
                tokio::join!(clear_sessions(), convert_goals(&jobs_pool, &time));
        }
    })
}

async fn inject_context(mut request: Request, next: Next) -> Response {
    let handlebars_context = HandlebarsContext::new();
    request.extensions_mut().insert(handlebars_context);

    next.run(request).await
}

async fn healthcheck() -> AppResponse {
    Ok(StatusCode::OK.into_response())
}

async fn cache_assets(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;

    if response.status().is_success() {
        response.headers_mut().insert(
            "Cache-Control",
            HeaderValue::from_static("public, max-age=31536000"),
        );
    }

    response
}

#[tokio::main]
async fn main() {
    // Reads endpoints and log level from environment variables
    let mut telemetry = TelemetryBuilder::new("simple-budget".to_string());
    telemetry.init().expect("could not initialize subscriber");

    let mut handlebars = Handlebars::new();
    handlebars.set_dev_mode(true);
    handlebars.set_strict_mode(true);
    handlebars.register_helper(
        "digest_asset",
        Box::new(DigestAssetHandlebarsHelper {
            key: Utc::now().timestamp_millis().to_string(),
        }),
    );
    handlebars.register_helper("eq", Box::new(EqHandlebarsHelper {}));

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

    let secret_key = env::var("SECRET_KEY").expect("cannot find secret key");
    let key = Key::from(secret_key.as_bytes());

    let pool = database_pool(None).await.unwrap();

    let shared_state = SharedState {
        key,
        pool,
        handlebars,
    };

    let app = Router::new()
        .merge(authentication::authentication_router())
        .merge(authenticated::authenticated_router(shared_state.clone()))
        .merge(Router::new().route("/healthcheck", get(healthcheck)))
        .with_state(shared_state)
        .layer(from_fn(inject_context))
        .merge(
            Router::new()
                .nest_service("/assets", ServeDir::new("static"))
                .layer(from_fn(cache_assets)),
        )
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();

    let server_handle = spawn(async {
        axum::serve(listener, app).await.unwrap();
    });
    let background_jobs = start_background_jobs();

    let mut signal = signal(SignalKind::terminate()).unwrap();

    let signal_listener = spawn(async move {
        signal.recv().await;
        debug!("received SIGTERM");
        0
    });

    select! {
        _ = signal_listener => {},
        _ = background_jobs => {},
        _ = server_handle => {},
    }
}

pub async fn database_pool(database_url: Option<&str>) -> Result<Pool> {
    let database_url = match database_url {
        Some(url) => url,
        None => &env::var("DATABASE_URL")?,
    };

    let mut cfg = Config::new();
    cfg.url = Some(database_url.to_string());
    cfg.manager = Some(deadpool_postgres::ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });

    Ok(cfg.create_pool(Some(Runtime::Tokio1), NoTls)?)
}

pub async fn database_client(database_url: Option<&str>) -> Result<Client> {
    let database_url = match database_url {
        Some(url) => url,
        None => &env::var("DATABASE_URL")?,
    };
    let (client, connection) = tokio_postgres::connect(database_url, NoTls).await?;

    spawn(connection);
    Ok(client)
}

#[cfg(test)]
mod test_utils;

#[cfg(test)]
mod tests {
    use crate::database_client;

    #[tokio::test]
    async fn test_database_client() {
        let client = database_client(Some(
            "postgres://simple_budget@localhost:5432/simple_budget_test",
        ))
        .await;
        assert!(client.is_ok());
        let client = client.unwrap();
        let row = client.execute("SELECT 1", &[]).await;
        assert!(row.is_ok());
    }
}
