use crate::{LayoutCtx, errors::AppResponse, utilities::responses::generate_response};
use askama::Template;
use axum::{Extension, http::StatusCode};

#[derive(Template)]
#[template(path = "accounts/new.html")]
struct New {
    #[allow(unused)]
    layout: LayoutCtx,

    name: String,
    amount: String,
    errors: String,
    debt: bool,
}

pub async fn action(Extension(layout): Extension<LayoutCtx>) -> AppResponse {
    let new = New {
        layout,
        name: String::new(),
        amount: String::new(),
        errors: String::new(),
        debt: false,
    };
    Ok(generate_response(
        &crate::utilities::responses::ResponseFormat::Html,
        new.render()?,
        StatusCode::OK,
    ))
}
