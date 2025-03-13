use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use tracing::error;

#[derive(Debug)]
pub enum AppError {
    RecordNotFound(tokio_postgres::Error),
    RecordDeserializationError(tokio_postgres::Error),
    Unknown(anyhow::Error),
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(value: E) -> Self {
        Self::Unknown(value.into())
    }
}

pub type AppResponse = Result<Response, AppError>;

impl IntoResponse for AppError {
    fn into_response(self: AppError) -> Response {
        match self {
            AppError::RecordNotFound(err) => {
                error!("{}", err);
                StatusCode::NOT_FOUND.into_response()
            }
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Sorry, something has gone wrong",
            )
                .into_response(),
        }
    }
}
