use anyhow::Result;
use axum::{
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
};

/// Represents the type of response format requested
#[derive(Debug, PartialEq)]
pub enum ResponseFormat {
    Html,
    Turbo,
    Json,
}

/// Determines the response format based on request headers
pub fn get_response_format(headers: &HeaderMap) -> Result<ResponseFormat> {
    if let Some(accept) = headers.get("Accept") {
        let accept_str = accept.to_str()?;
        if accept_str.contains("turbo") {
            Ok(ResponseFormat::Turbo)
        } else if accept_str.contains("application/json") {
            Ok(ResponseFormat::Json)
        } else {
            Ok(ResponseFormat::Html)
        }
    } else {
        Ok(ResponseFormat::Html)
    }
}

/// Creates a response for an invalid form submission with appropriate headers and status
pub fn generate_response<T: IntoResponse>(
    format: &ResponseFormat,
    content: T,
    status: StatusCode,
) -> Response {
    match format {
        ResponseFormat::Turbo => (
            status,
            [("content-type", "text/vnd.turbo-stream.html")],
            Html::from(content),
        )
            .into_response(),
        ResponseFormat::Json => {
            (status, [("content-type", "application/json")], content).into_response()
        }
        ResponseFormat::Html => (status, Html::from(content)).into_response(),
    }
}
