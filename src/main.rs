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
use simple_logger::SimpleLogger;
use std::{env, str::FromStr};
use tera::{Context, Tera};
mod authenticated;

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

async fn root(shared_state: State<SharedState>) -> Response {
    let context = Context::new();
    let Ok(content) = shared_state.tera.render("dashboard.html", &context) else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    Html::from(content).into_response()
}

#[tokio::main]
async fn main() {
    let _ = SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
        .init()
        .expect("could not initialize logging");

    let tera = Tera::new("src/templates/**/*.html").expect("could not initialize Tera");
    let mongo = mongo_client().await.expect("could not create Mongo client");

    let key = Key::generate();
    let shared_state = SharedState { tera, mongo, key };

    let app = Router::new()
        .nest("/authentication", authentication::authentication_router())
        .nest(
            "/",
            authenticated::authenticated_router(shared_state.clone()),
        )
        .route("/", get(root))
        .with_state(shared_state);

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
