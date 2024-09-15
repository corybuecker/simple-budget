mod authentication;
mod errors;
mod jobs;
use axum::{extract::FromRef, Router};
use axum_extra::extract::cookie::Key;
use bson::doc;
use chrono::{Local, Utc};
use models::user::User;
use mongodb::Client;
use std::{
    collections::HashMap,
    env,
    time::{Duration, SystemTime},
};
use tera::Tera;
use tokio::{
    select,
    signal::unix::{signal, SignalKind},
    spawn,
    sync::mpsc,
    time::interval,
};
mod authenticated;
use tower_http::{
    services::ServeDir,
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};
use tracing::{debug, Level};
mod models;

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

pub fn digest_asset() -> impl tera::Function {
    let key = SystemTime::now();
    let key = key
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("could not generate asset timestamp");
    let key = key.as_secs().to_string();

    return move |args: &HashMap<String, tera::Value>| -> tera::Result<tera::Value> {
        match args.get("file") {
            Some(file) => {
                let mut path = "/assets/".to_string();

                let Some(file) = file.as_str() else {
                    return Err("".to_string().into());
                };

                path.push_str(file);
                path.push_str("?v=");
                path.push_str(&key);

                Ok(path.into())
            }
            None => Err("".to_string().into()),
        }
    };
}

async fn current_time() {
    println!("{}", Local::now())
}

async fn clear_sessions() {
    let mongo = mongo_client().await.unwrap();

    let update_result = mongo
        .default_database()
        .unwrap()
        .collection::<User>("users")
        .update_many(
            doc! {},
            doc! {"$pull": doc! {"sessions": doc! {"expiration": doc! { "$lte": Utc::now()}}}},
        )
        .await;

    match update_result {
        Ok(_) => {}
        Err(err) => {
            println!("{}", err);
        }
    }
}

async fn convert_goals_wrapper() {
    let mongo = mongo_client().await.unwrap();
    let _ = jobs::convert_goals::convert_goals(mongo.start_session().await.unwrap()).await;
}

fn start_background_jobs() -> tokio::task::JoinHandle<()> {
    spawn(async {
        let mut interval = interval(Duration::from_millis(60000));

        loop {
            let h1 = async { current_time().await };
            let h2 = async { current_time().await };

            interval.tick().await;

            tokio::join!(h1, h2, clear_sessions(), convert_goals_wrapper());
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

#[tokio::main]
async fn main() {
    let tracing_fmt = tracing_subscriber::fmt::format().pretty();
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .event_format(tracing_fmt)
        .init();

    let mut tera = Tera::new("src/templates/**/*.html").expect("cannot initialize Tera");
    tera.register_function("digest_asset", digest_asset());

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
        .nest_service("/assets", ServeDir::new("static"))
        .with_state(shared_state)
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
