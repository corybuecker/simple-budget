mod authentication;
use axum::{extract::FromRef, Router};
use axum_extra::extract::cookie::Key;
use bson::{doc, oid::ObjectId};
use chrono::{DateTime, Local, Utc};
use mongodb::Client;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env,
    str::FromStr,
    thread,
    time::{Duration, SystemTime},
};
use tera::Tera;
mod authenticated;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing::Level;
mod models;

#[derive(Clone)]
struct SharedState {
    tera: Tera,
    mongo: Client,
    key: Key,
}

impl FromRef<SharedState> for Key {
    fn from_ref(state: &SharedState) -> Self {
        state.key.clone()
    }
}

// struct AssetDigests {}

// impl tera::Function for AssetDigests {
//     fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
//         Ok("test".to_string().into())
//     }
// }

fn currency() -> impl tera::Function {
    return move |args: &HashMap<String, tera::Value>| -> tera::Result<tera::Value> {
        match args.get("number") {
            Some(number) => Ok(number.clone()),
            None => Err("".to_string().into()),
        }
    };
}
fn digest_asset() -> impl tera::Function {
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

                log::debug!("{:?}", path);

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
        .database("simple_budget")
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

fn start_background_jobs() -> tokio::task::JoinHandle<()> {
    tokio::spawn(async {
        loop {
            let h1 = async { current_time().await };
            let h2 = async { current_time().await };

            tokio::join!(h1, h2, clear_sessions());

            thread::sleep(Duration::from_millis(5000))
        }
    })
}

#[tokio::main]
async fn main() {
    let sentry_url = env::var("SENTRY_URL").unwrap();
    let _guard = sentry::init((
        sentry_url,
        sentry::ClientOptions {
            release: sentry::release_name!(),
            ..Default::default()
        },
    ));

    let tracing_fmt = tracing_subscriber::fmt::format().pretty();
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .event_format(tracing_fmt)
        .init();

    let mut tera = Tera::new("src/templates/**/*.html").expect("cannot initialize Tera");
    tera.register_function("digest_asset", digest_asset());
    tera.register_function("currency", currency());

    let mongo = mongo_client().await.expect("cannot create Mongo client");

    let secret_key = env::var("SECRET_KEY").expect("cannot find secret key");
    let key = Key::from(secret_key.as_bytes());

    let shared_state = SharedState { tera, mongo, key };
    let app = Router::new()
        .nest("/authentication", authentication::authentication_router())
        .nest(
            "/",
            authenticated::authenticated_router(shared_state.clone()),
        )
        .nest_service("/assets", ServeDir::new("static"))
        .with_state(shared_state)
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();

    let server_handle = tokio::spawn(async {
        axum::serve(listener, app).await.unwrap();
    });

    let background_jobs_handle = start_background_jobs();

    match tokio::try_join!(server_handle, background_jobs_handle) {
        Ok(_) => {}
        Err(err) => {
            println!("{}", err);
        }
    };

    return ();
}

async fn mongo_client() -> Result<mongodb::Client, mongodb::error::Error> {
    let mongo_connection_string = env::var("DATABASE_URL").unwrap_or(
        String::from_str("mongodb://localhost:27017/simple_budget?connectTimeoutMS=1000").unwrap(),
    );

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
