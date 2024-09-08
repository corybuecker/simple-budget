mod authentication;
mod errors;
mod jobs;
mod test_utils;
use axum::{extract::FromRef, Router};
use axum_extra::extract::cookie::Key;
use bson::{doc, oid::ObjectId};
use chrono::{DateTime, Local, Utc};
use mongodb::Client;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env, thread,
    time::{Duration, SystemTime},
};
use tera::Tera;
use tokio::sync::mpsc;
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
struct SharedState {
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

#[derive(Deserialize, Serialize)]
struct Session {
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    expiration: DateTime<Utc>,
    id: bson::Uuid,
    csrf: String,
}

#[derive(Deserialize, Serialize)]
struct User {
    subject: String,
    email: String,
    sessions: Vec<Session>,
    _id: ObjectId,
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
    tokio::spawn(async {
        loop {
            let h1 = async { current_time().await };
            let h2 = async { current_time().await };

            tokio::join!(h1, h2, clear_sessions(), convert_goals_wrapper());

            thread::sleep(Duration::from_millis(60000))
        }
    })
}

fn start_broker(mut receiver: mpsc::Receiver<String>) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
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

    let server_handle = tokio::spawn(async {
        axum::serve(listener, app).await.unwrap();
    });
    let background_jobs_handle = start_background_jobs();
    let broker_handle = start_broker(receiver);

    match tokio::try_join!(server_handle, background_jobs_handle, broker_handle) {
        Ok(_) => {}
        Err(err) => {
            println!("{}", err);
        }
    };

    return;
}

async fn mongo_client() -> Result<mongodb::Client, mongodb::error::Error> {
    let mongo_connection_string =
        env::var("DATABASE_URL").expect("could not find database connection URL");

    Client::with_uri_str(mongo_connection_string).await
}

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
