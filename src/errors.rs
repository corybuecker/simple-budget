use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use tracing::error;

#[derive(Debug)]
pub enum AppError {
    InvalidRecord(anyhow::Error),
    RecordDeserializationError(tokio_postgres::Error),
    RecordNotFound(tokio_postgres::Error),
    TemplateError(handlebars::RenderError),
    Unknown(anyhow::Error),
}

impl From<handlebars::RenderError> for AppError {
    fn from(value: handlebars::RenderError) -> Self {
        Self::TemplateError(value)
    }
}

impl From<tokio_postgres::Error> for AppError {
    fn from(value: tokio_postgres::Error) -> Self {
        Self::Unknown(value.into())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(value: serde_json::Error) -> Self {
        Self::Unknown(value.into())
    }
}

impl From<deadpool_postgres::PoolError> for AppError {
    fn from(value: deadpool_postgres::PoolError) -> Self {
        Self::Unknown(value.into())
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(value: jsonwebtoken::errors::Error) -> Self {
        Self::Unknown(value.into())
    }
}

impl From<chrono_tz::ParseError> for AppError {
    fn from(value: chrono_tz::ParseError) -> Self {
        Self::Unknown(value.into())
    }
}

impl From<openidconnect::url::ParseError> for AppError {
    fn from(value: openidconnect::url::ParseError) -> Self {
        Self::Unknown(value.into())
    }
}

impl From<openidconnect::reqwest::Error> for AppError {
    fn from(value: openidconnect::reqwest::Error) -> Self {
        Self::Unknown(value.into())
    }
}

impl
    From<
        openidconnect::DiscoveryError<
            openidconnect::HttpClientError<openidconnect::reqwest::Error>,
        >,
    > for AppError
{
    fn from(
        value: openidconnect::DiscoveryError<
            openidconnect::HttpClientError<openidconnect::reqwest::Error>,
        >,
    ) -> Self {
        Self::Unknown(value.into())
    }
}

impl From<std::env::VarError> for AppError {
    fn from(value: std::env::VarError) -> Self {
        Self::Unknown(value.into())
    }
}

impl From<anyhow::Error> for AppError {
    fn from(value: anyhow::Error) -> Self {
        Self::Unknown(value)
    }
}

pub type AppResponse = Result<Response, AppError>;

impl IntoResponse for AppError {
    fn into_response(self: AppError) -> Response {
        match self {
            AppError::InvalidRecord(err) => {
                error!("{}", err);
                StatusCode::BAD_REQUEST.into_response()
            }
            AppError::RecordNotFound(err) => {
                error!("{}", err);
                StatusCode::NOT_FOUND.into_response()
            }
            _ => {
                println!("Unhandled error: {:?}", self);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Sorry, something has gone wrong",
                )
                    .into_response()
            }
        }
    }
}
