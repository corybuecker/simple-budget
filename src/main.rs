mod authenticated;
mod authentication;
mod db;
mod errors;
mod jobs;
mod middleware;
mod models;
mod templates;
mod utilities;

use axum::{
    Router, extract::FromRef, http::StatusCode, middleware::from_fn, response::IntoResponse,
    routing::get,
};
use axum_extra::extract::cookie::Key;
use errors::AppResponse;
use jobs::{clear_sessions::clear_sessions, convert_goals::convert_goals};
use rust_database_common::DatabasePool;
use serde::Serialize;
use std::{env, fs, sync::OnceLock, time::Duration};
use tokio::{
    select,
    signal::unix::{SignalKind, signal},
    spawn,
    time::interval,
};
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing::{Level, debug};
use utilities::dates::TimeProvider;

#[derive(Serialize, Clone, Debug)]
pub enum Section {
    Reports,
    Accounts,
    Envelopes,
    Goals,
    Preferences,
}

#[derive(Clone, Debug)]
pub struct LayoutCtx {
    pub section: Section,
    pub csrf_token: Option<String>,
    pub nonce: String,
}
impl LayoutCtx {
    /// Resolves a Vite entry (e.g. `assets/index.ts`) to its built, hashed path.
    /// The manifest is read once; if it is missing (e.g. in tests, before a
    /// frontend build) the entry name is served as-is.
    pub fn asset(&self, filename: &str) -> String {
        static MANIFEST: OnceLock<serde_json::Value> = OnceLock::new();

        let manifest = MANIFEST.get_or_init(|| {
            fs::read_to_string("static/.vite/manifest.json")
                .ok()
                .and_then(|raw| serde_json::from_str(&raw).ok())
                .unwrap_or(serde_json::Value::Null)
        });

        match manifest[filename]["file"].as_str() {
            Some(file) => format!("/assets/{}", file),
            None => format!("/assets/{}", filename),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SharedState {
    key: Key,
    pool: DatabasePool,
}

impl FromRef<SharedState> for Key {
    fn from_ref(state: &SharedState) -> Self {
        state.key.clone()
    }
}

fn start_background_jobs() -> tokio::task::JoinHandle<()> {
    spawn(async {
        let mut interval = interval(Duration::from_millis(60000));
        let database_pool = db::database_pool(None).await.unwrap();

        let time = TimeProvider {};

        loop {
            interval.tick().await;

            let (_clear_sessions_result, _convert_goals_result) = tokio::join!(
                clear_sessions(&database_pool),
                convert_goals(&database_pool, &time)
            );
        }
    })
}

async fn healthcheck() -> AppResponse {
    Ok(StatusCode::OK.into_response())
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    let secret_key = env::var("SECRET_KEY").expect("cannot find secret key");
    let key = Key::from(secret_key.as_bytes());

    let pool = match db::database_pool(None).await {
        Ok(pool) => pool,
        Err(err) => {
            panic!("failed to connect to database: {:#?}", err);
        }
    };

    let shared_state = SharedState { key, pool };

    let app = Router::new()
        .merge(authentication::authentication_router())
        .merge(authenticated::authenticated_router(shared_state.clone()))
        .merge(Router::new().route("/healthcheck", get(healthcheck)))
        .with_state(shared_state)
        .layer(from_fn(middleware::secure_headers))
        .layer(from_fn(middleware::inject_layout_context))
        .merge(
            Router::new()
                .nest_service("/assets", ServeDir::new("static"))
                .layer(from_fn(middleware::cache_assets)),
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

#[cfg(test)]
mod test_utils;
