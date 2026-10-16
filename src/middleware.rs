use crate::{LayoutCtx, errors::AppError};
use anyhow::anyhow;
use axum::{extract::Request, http::HeaderValue, middleware::Next, response::Response};
use uuid::Uuid;

pub async fn inject_layout_context(mut request: Request, next: Next) -> Response {
    let layout_context = LayoutCtx {
        nonce: Uuid::new_v4().to_string(),
        section: crate::Section::Reports,
        csrf_token: None,
    };

    request.extensions_mut().insert(layout_context);

    next.run(request).await
}

pub async fn secure_headers(request: Request, next: Next) -> Result<Response, AppError> {
    let nonce = {
        let context = request
            .extensions()
            .get::<LayoutCtx>()
            .ok_or(AppError::Unknown(anyhow!("missing nonce value")))?;

        context.nonce.to_string()
    };

    let mut response = next.run(request).await;

    response.headers_mut().insert(
        "Content-Security-Policy",
        HeaderValue::from_str(&format!(
            "default-src 'none'; script-src 'nonce-{}'; style-src 'nonce-{}'; img-src 'self'; connect-src 'self'",
            nonce, nonce
        )).unwrap());

    Ok(response)
}

pub async fn cache_assets(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;

    if response.status().is_success() {
        response.headers_mut().insert(
            "Cache-Control",
            HeaderValue::from_static("public, max-age=31536000"),
        );
    }

    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode},
        middleware::from_fn,
        routing::get,
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_csp_header_applied() {
        let app = Router::new()
            .route("/test", get(|| async { "test response" }))
            .layer(from_fn(secure_headers))
            .layer(from_fn(inject_layout_context));

        let response = app
            .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let csp_header = response.headers().get("Content-Security-Policy");
        assert!(csp_header.is_some());

        let csp_value = csp_header.unwrap().to_str().unwrap();
        assert!(csp_value.contains("default-src 'none'"));
        assert!(csp_value.contains("script-src 'nonce-"));
        assert!(csp_value.contains("style-src 'nonce-"));
        assert!(csp_value.contains("img-src 'self'"));
        assert!(csp_value.contains("connect-src 'self'"));
    }

    #[tokio::test]
    async fn test_nonce_injection() {
        let app = Router::new()
            .route(
                "/test",
                get(|request: Request<Body>| async move {
                    let context = request.extensions().get::<LayoutCtx>().unwrap();
                    let nonce = &context.nonce;
                    assert!(!nonce.is_empty());
                    "ok"
                }),
            )
            .layer(from_fn(inject_layout_context));

        let response = app
            .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
