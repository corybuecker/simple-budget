mod authentication;
use axum::{extract::FromRef, routing::get, Router};
use axum_extra::extract::cookie::Key;
use mongodb::Client;
use simple_logger::SimpleLogger;
use std::{env, str::FromStr};
use tera::Tera;
mod api;
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

async fn root() -> &'static str {
    "Hello, World!"
}

#[tokio::main]
async fn main() {
    let _ = SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
        .init()
        .expect("could not initialize logging");

    let tera = Tera::new("src/templates/**/*.html").expect("could not initialize Tera");
    let mongo_connection_string = env::var("DATABASE_URL")
        .unwrap_or(String::from_str("mongodb://localhost:27017/simple_budget").unwrap());
    let mongo = Client::with_uri_str(mongo_connection_string)
        .await
        .expect("could not create Mongo client");

    let key = Key::generate();
    let shared_state = SharedState { tera, mongo, key };

    let app = Router::new()
        .nest("/authentication", authentication::authentication_router())
        .nest("/api", api::api_router(shared_state.clone()))
        .route("/", get(root))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
