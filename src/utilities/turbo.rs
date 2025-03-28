use anyhow::Result;
use axum::{
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
};

/// Checks if the request headers contain the turbo header
pub fn is_turbo_request(headers: &HeaderMap) -> Result<bool> {
    if let Some(accept) = headers.get("Accept") {
        Ok(accept.to_str()?.contains("turbo"))
    } else {
        Ok(false)
    }
}

/// Returns the appropriate template name based on whether this is a turbo request
pub fn get_template_name(is_turbo: bool, resource: &str, template_type: &str) -> String {
    if is_turbo {
        format!("{}/{}.turbo.html", resource, template_type)
    } else {
        format!("{}/{}.html", resource, if template_type == "form" { template_type } else { "new" })
    }
}

/// Creates a response for an invalid form submission with appropriate headers and status
pub fn form_error_response(
    is_turbo: bool, 
    content: String,
    status: StatusCode,
) -> Response {
    if is_turbo {
        (
            status,
            [("content-type", "text/vnd.turbo-stream.html")],
            Html::from(content),
        )
            .into_response()
    } else {
        (status, Html::from(content)).into_response()
    }
}