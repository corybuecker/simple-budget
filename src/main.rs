mod authenticated;
mod authentication;
mod db;
mod errors;
mod jobs;
mod middleware;
mod models;
mod utilities;

use crate::utilities::handlebars::{DigestAssetHandlebarsHelper, EqHandlebarsHelper, walk_directory};
use axum::{
    Router,
    extract::FromRef,
    http::StatusCode,
    middleware::from_fn,
    response::IntoResponse,
    routing::get,
};
use axum_extra::extract::cookie::Key;
use chrono::Utc;
use errors::AppResponse;
use handlebars::Handlebars;
use jobs::{clear_sessions::clear_sessions, convert_goals::convert_goals};
use rust_database_common::DatabasePool;
use rust_web_common::telemetry::TelemetryBuilder;
use serde::Serialize;
use std::{collections::BTreeMap, env, time::Duration};
use tokio::{
    select,
    signal::unix::{SignalKind, signal},
    spawn,
    time::interval,
};
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
    pool: DatabasePool,
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
    let mut telemetry = TelemetryBuilder::new("simple-budget".to_string()).with_json_log_format();
    telemetry.init().expect("could not initialize subscriber");

    let cache_key = Utc::now().timestamp_millis().to_string();
    let mut handlebars = Handlebars::new();
    handlebars.set_dev_mode(true);
    handlebars.set_strict_mode(true);
    handlebars.register_helper(
        "digest_asset",
        Box::new(DigestAssetHandlebarsHelper {
            key: cache_key.clone(),
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

    let pool = match db::database_pool(None).await {
        Ok(pool) => pool,
        Err(err) => {
            panic!("failed to connect to database: {:#?}", err);
        }
    };

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
        .layer(from_fn(middleware::secure_headers))
        .layer(from_fn(middleware::inject_context))
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
