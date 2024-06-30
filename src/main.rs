mod authentication;
use axum::{
    extract::{FromRef, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use axum_extra::extract::cookie::Key;
use mongodb::Client;
use std::{collections::HashMap, env, str::FromStr, time::SystemTime};
use tera::{Context, Tera};
use tower_http::services::ServeDir;
mod authenticated;
use tower_http::trace::{self, TraceLayer};
use tracing::Level;

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

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    let mut tera = Tera::new("src/templates/**/*.html").expect("cannot initialize Tera");
    tera.register_function("digest_asset", digest_asset());

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
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
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
        let databases = client.list_databases(None, None).await;
        assert!(databases.is_ok())
    }
}
