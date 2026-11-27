use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

/// Wraps rendered HTML in a response with the given status.
pub fn generate_response(content: String, status: StatusCode) -> Response {
    (status, Html::from(content)).into_response()
}
