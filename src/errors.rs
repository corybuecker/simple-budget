use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

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

impl From<bson::oid::Error> for FormError {
    fn from(value: bson::oid::Error) -> Self {
        FormError {
            message: value.to_string(),
            status_code: None,
        }
    }
}

impl From<tera::Error> for FormError {
    fn from(value: tera::Error) -> Self {
        log::error!("{:#?}", value);

        FormError {
            message: value.to_string(),
            status_code: None,
        }
    }
}

impl From<mongodb::error::Error> for FormError {
    fn from(value: mongodb::error::Error) -> Self {
        FormError {
            message: value.to_string(),
            status_code: None,
        }
    }
}

impl From<ModelError> for FormError {
    fn from(value: ModelError) -> Self {
        log::error!("{:#?}", value);

        FormError {
            message: value.to_string(),
            status_code: None,
        }
    }
}

#[derive(Debug)]
pub enum ModelError {
    MissingDefaultDatabase,

    #[allow(dead_code)]
    OidError(bson::oid::Error),

    #[allow(dead_code)]
    OidParsingError(bson::oid::Error),

    #[allow(dead_code)]
    DatabaseError(mongodb::error::Error),
}

impl From<bson::oid::Error> for ModelError {
    fn from(err: bson::oid::Error) -> ModelError {
        ModelError::OidError(err)
    }
}
impl From<mongodb::error::Error> for ModelError {
    fn from(err: mongodb::error::Error) -> ModelError {
        ModelError::DatabaseError(err)
    }
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
            Self::OidError(bson_error) => write!(f, "{}", bson_error),
            Self::OidParsingError(bson_error) => write!(f, "{}", bson_error),
            Self::DatabaseError(mongo_error) => write!(f, "{}", mongo_error),
        }
    }
}
