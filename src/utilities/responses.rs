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

/// Returns the appropriate template name based on the response format
pub fn get_template_name(format: &ResponseFormat, resource: &str, template_type: &str) -> String {
    match format {
        ResponseFormat::Turbo => format!("{resource}/{template_type}.turbo.html"),
        ResponseFormat::Json => format!("{resource}/{template_type}.json"),
        ResponseFormat::Html => format!(
            "{}/{}.html",
            resource,
            if template_type == "form"
                || template_type == "index"
                || template_type == "delete"
                || template_type == "edit"
                || template_type == "new"
            {
                template_type
            } else {
                "new"
            }
        ),
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
