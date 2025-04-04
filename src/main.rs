use anyhow::Result;
use axum::{
    Router,
    body::Body,
    extract::{FromRef, Path, Request},
    http::{HeaderValue, StatusCode},
    middleware::{Next, from_fn},
    response::{IntoResponse, Response},
    routing::get,
};
use axum_extra::extract::cookie::Key;
use deadpool_postgres::{Config, Pool, RecyclingMethod, Runtime};
use errors::AppResponse;
use include_dir::{Dir, include_dir};
use jobs::{clear_sessions::clear_sessions, convert_goals::convert_goals};
use opentelemetry::{KeyValue, global};
use serde::Serialize;
use std::{env, time::Duration};
use tera::{Context, Tera};
use tokio::{
    select,
    signal::unix::{SignalKind, signal},
    spawn,
    sync::mpsc,
    time::{Instant, interval},
};
use tokio_postgres::{Client, NoTls};
use tower_http::trace::TraceLayer;
use tracing::debug;
use utilities::{dates::TimeProvider, initialize_logging, tera::digest_asset};

mod authenticated;
mod authentication;
mod errors;
mod jobs;
mod models;
mod utilities;

#[derive(Serialize, Clone)]
pub enum Section {
    Reports,
    Accounts,
    Envelopes,
    Goals,
    Preferences,
}

#[derive(Clone)]
pub struct ContextExtension {
    pub context: Context,
}

#[derive(Debug, Clone)]
struct Broker {
    sender: mpsc::Sender<String>,
}

#[derive(Clone, Debug)]
pub struct SharedState {
    broker: Broker,
    key: Key,
    pool: Pool,
    tera: Tera,
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

fn start_broker(mut receiver: mpsc::Receiver<String>) -> tokio::task::JoinHandle<()> {
    spawn(async move {
        loop {
            if let Some(message) = receiver.recv().await {
                debug!("{}", message);
            }
        }
    })
}

async fn inject_context(mut request: Request, next: Next) -> Response {
    let context = Context::new();
    let context_extension = ContextExtension { context };

    request.extensions_mut().insert(context_extension);

    next.run(request).await
}

async fn cache_header(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert(
        "cache-control",
        HeaderValue::from_str("public, max-age=31536000").unwrap(),
    );
    response
}

async fn fetch_asset(Path(file): Path<String>) -> Response {
    debug!("{}", file);

    let mut content_type: &str = "text/plain";

    if file.ends_with(".png") {
        content_type = "image/png"
    };

    if file.ends_with(".js") {
        content_type = "application/javascript"
    };

    if file.ends_with(".css") {
        content_type = "text/css"
    };

    match ASSETS.get_file(&file) {
        Some(asset) => {
            let mut response = Body::from(asset.contents()).into_response();
            let headers = response.headers_mut();
            headers.insert("content-type", HeaderValue::from_str(content_type).unwrap());
            response
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn healthcheck() -> AppResponse {
    Ok(StatusCode::OK.into_response())
}

async fn capture_metrics(request: Request, next: Next) -> Response {
    let path = (&request.uri().path()).to_string();
    let method = request.method().as_str().to_string();
    let meter = global::meter("simple_budget");
    let counter = meter.u64_counter("requests").build();
    let latency = meter.f64_gauge("latency").build();
    counter.add(
        1,
        &[
            KeyValue::new("method", method.clone()),
            KeyValue::new("path", path.clone()),
            KeyValue::new("app", "simple-budget"),
        ],
    );

    let start = Instant::now();
    let response = next.run(request).await;
    latency.record(
        start.elapsed().as_secs_f64(),
        &[
            KeyValue::new("method", method.clone()),
            KeyValue::new("path", path.clone()),
            KeyValue::new("app", "simple-budget"),
        ],
    );
    response
}

static TEMPLATES: Dir = include_dir!("templates");
static ASSETS: Dir = include_dir!("static");

#[tokio::main]
async fn main() {
    initialize_logging();

    let mut tera = Tera::default();
    tera.register_function("digest_asset", digest_asset());

    for template in TEMPLATES.find("**/*.html").unwrap() {
        debug!("{:#?}", template);
        let _ = tera.add_raw_template(
            template.path().to_str().unwrap(),
            TEMPLATES
                .get_file(template.path())
                .unwrap()
                .contents_utf8()
                .unwrap(),
        );
    }

    let secret_key = env::var("SECRET_KEY").expect("cannot find secret key");
    let key = Key::from(secret_key.as_bytes());
    let (sender, receiver) = mpsc::channel::<String>(100);
    let pool = database_pool(None).await.unwrap();

    let shared_state = SharedState {
        tera,
        key,
        pool,
        broker: Broker { sender },
    };

    let app = Router::new()
        .merge(authentication::authentication_router())
        .merge(authenticated::authenticated_router(shared_state.clone()))
        .merge(
            Router::new()
                .route("/assets/{*file}", get(fetch_asset))
                .layer(from_fn(cache_header)),
        )
        .merge(Router::new().route("/healthcheck", get(healthcheck)))
        .with_state(shared_state)
        .layer(from_fn(inject_context))
        .layer(TraceLayer::new_for_http())
        .layer(from_fn(capture_metrics));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();

    let server_handle = spawn(async {
        axum::serve(listener, app).await.unwrap();
    });
    let background_jobs = start_background_jobs();
    let broker_handle = start_broker(receiver);

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
        _ = broker_handle => {},
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
