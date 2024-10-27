mod authentication;
mod errors;
mod jobs;
mod utilities;
use axum::{
    body::Body,
    extract::{FromRef, Path, Request},
    http::{HeaderValue, StatusCode},
    middleware::{from_fn, Next},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use axum_extra::extract::cookie::Key;
use bson::doc;
use include_dir::{include_dir, Dir};
use jobs::{clear_sessions::clear_sessions, convert_goals::convert_goals};
use mongodb::Client;
use serde::Serialize;
use std::{env, time::Duration};
use tera::{Context, Tera};
use tokio::{
    select,
    signal::unix::{signal, SignalKind},
    spawn,
    sync::mpsc,
    time::interval,
};
use utilities::tera::{digest_asset, extract_id};
mod authenticated;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::{debug, Level};
mod models;

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
    mongo: Client,
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

        loop {
            interval.tick().await;

            let _result = tokio::join!(clear_sessions(), convert_goals());
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
    tera.register_filter("oid", extract_id());

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

    let mongo = mongo_client().await.expect("cannot create Mongo client");

    let secret_key = env::var("SECRET_KEY").expect("cannot find secret key");
    let key = Key::from(secret_key.as_bytes());
    let (sender, receiver) = mpsc::channel::<String>(100);

    let shared_state = SharedState {
        tera,
        mongo,
        key,
        broker: Broker { sender },
    };

    let app = Router::new()
        .nest("/authentication", authentication::authentication_router())
        .nest(
            "/",
            authenticated::authenticated_router(shared_state.clone()),
        )
        .nest(
            "/assets",
            Router::new()
                .route("/*file", get(fetch_asset))
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

async fn mongo_client() -> Result<mongodb::Client, mongodb::error::Error> {
    let mongo_connection_string =
        env::var("DATABASE_URL").expect("could not find database connection URL");

    Client::with_uri_str(mongo_connection_string).await
}

#[cfg(test)]
mod test_utils;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mongo_client() {
        let client = mongo_client().await;
        assert!(client.is_ok());
        let client = client.unwrap();
        let databases = client.list_databases().await;
        assert!(databases.is_ok())
    }
}
