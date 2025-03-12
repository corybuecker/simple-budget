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
use include_dir::{Dir, include_dir};
use jobs::{clear_sessions::clear_sessions, convert_goals::convert_goals};
use serde::Serialize;
use std::{env, sync::Arc, time::Duration};
use tera::{Context, Tera};
use tokio::{
    select,
    signal::unix::{SignalKind, signal},
    spawn,
    sync::mpsc,
    time::interval,
};
use tokio_postgres::{Client, NoTls};
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::{Level, debug};
use utilities::tera::digest_asset;

mod authenticated;
mod authentication;
mod errors;
mod jobs;
mod models;
mod utilities;

#[derive(Serialize, Debug, Clone)]
pub enum Section {
    Reports,
    Accounts,
    Envelopes,
    Goals,
    Preferences,
}

#[derive(Debug, Clone)]
pub struct ContextExtension {
    pub context: Context,
}

#[derive(Clone)]
struct Broker {
    sender: mpsc::Sender<String>,
}

#[derive(Clone)]
pub struct SharedState {
    tera: Tera,
    client: Arc<Client>,
    key: Key,
    broker: Broker,
}

impl FromRef<SharedState> for Key {
    fn from_ref(state: &SharedState) -> Self {
        state.key.clone()
    }
}

fn start_background_jobs() -> tokio::task::JoinHandle<()> {
    spawn(async {
        let mut interval = interval(Duration::from_millis(60000));
        let mut jobs_client = database_client(None).await.unwrap();

        loop {
            interval.tick().await;

            let (clear_sessions_result, convert_goals_result) =
                tokio::join!(clear_sessions(), convert_goals(&mut jobs_client));

            debug!("🚧 {:#?}", convert_goals_result);
            debug!("🚧 {:#?}", clear_sessions_result);
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

static TEMPLATES: Dir = include_dir!("templates");
static ASSETS: Dir = include_dir!("static");

#[tokio::main]
async fn main() {
    let tracing_fmt = tracing_subscriber::fmt::format().pretty();
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .event_format(tracing_fmt)
        .init();

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
    let client = database_client(None).await.unwrap();
    let shared_state = SharedState {
        tera,
        client: client.into(),
        key,
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
        .with_state(shared_state)
        .layer(from_fn(inject_context))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(tracing::Level::INFO))
                .on_response(DefaultOnResponse::new().level(tracing::Level::INFO)),
        );

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
            "postgres://postgres@localhost:5432/simple_budget_test",
        ))
        .await;
        assert!(client.is_ok());
        let client = client.unwrap();
        let row = client.execute("SELECT 1", &[]).await;
        assert!(row.is_ok());
    }
}
