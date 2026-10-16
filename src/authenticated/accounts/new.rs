use super::NewTemplate;
use crate::{LayoutCtx, errors::AppResponse, utilities::responses::generate_response};
use askama::Template;
use axum::{Extension, http::StatusCode};

pub async fn action(Extension(layout): Extension<LayoutCtx>) -> AppResponse {
    let template = NewTemplate {
        layout,
        name: String::new(),
        amount: String::new(),
        debt: false,
        errors: String::new(),
    };

    Ok(generate_response(template.render()?, StatusCode::OK))
}
