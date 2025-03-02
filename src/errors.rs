use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use tracing::error;

#[allow(dead_code)]
#[derive(Debug)]
pub struct FormError {
    pub message: String,
    pub status_code: Option<StatusCode>,
}

impl IntoResponse for FormError {
    fn into_response(self) -> Response {
        (StatusCode::BAD_REQUEST, format!("{:#?}", self)).into_response()
    }
}

impl From<anyhow::Error> for FormError {
    fn from(value: anyhow::Error) -> Self {
        error!("{:#?}", value);

        FormError {
            message: value.to_string(),
            status_code: None,
        }
    }
}

impl From<tera::Error> for FormError {
    fn from(value: tera::Error) -> Self {
        error!("{:#?}", value);

        FormError {
            message: value.to_string(),
            status_code: None,
        }
    }
}

impl From<ModelError> for FormError {
    fn from(value: ModelError) -> Self {
        error!("{:#?}", value);

        FormError {
            message: value.to_string(),
            status_code: None,
        }
    }
}

#[derive(Debug)]
pub enum ModelError {
    #[allow(dead_code)]
    MissingDefaultDatabase,
}

impl std::fmt::Display for ModelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingDefaultDatabase => {
                write!(
                    f,
                    "default database not configured, check the connection string"
                )
            }
        }
    }
}
