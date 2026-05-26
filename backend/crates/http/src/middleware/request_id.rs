//! Request-id middleware.
//!
//! BACKEND.md §15: read `x-request-id` from the incoming request (or generate
//! a fresh UUIDv7), echo it on the response, and bind it as a
//! `tokio::task_local` so `AppError::into_response` can attach it to error
//! JSON. One middleware, one source of truth.

use axum::{
    extract::Request,
    http::{HeaderName, HeaderValue},
    middleware::Next,
    response::Response,
};
use common::request_id::with_request_id;
use uuid::Uuid;

pub const HEADER: HeaderName = HeaderName::from_static("x-request-id");

pub async fn layer(mut req: Request, next: Next) -> Response {
    let incoming = req
        .headers()
        .get(&HEADER)
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned);

    let request_id = incoming.unwrap_or_else(|| Uuid::now_v7().to_string());

    // Make it visible to downstream extractors and handlers.
    if let Ok(v) = HeaderValue::from_str(&request_id) {
        req.headers_mut().insert(&HEADER, v);
    }

    let id_for_resp = request_id.clone();
    let mut resp = with_request_id(request_id, next.run(req)).await;

    if let Ok(v) = HeaderValue::from_str(&id_for_resp) {
        resp.headers_mut().insert(&HEADER, v);
    }
    resp
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request, middleware, routing::get, Router};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    async fn echo_request_id() -> String {
        common::request_id::current_request_id().unwrap_or_default()
    }

    fn router() -> Router {
        Router::new()
            .route("/r", get(echo_request_id))
            .layer(middleware::from_fn(layer))
    }

    #[tokio::test]
    async fn generates_id_when_header_absent() {
        let req = Request::builder().uri("/r").body(Body::empty()).unwrap();
        let resp = router().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), 200);
        let echoed = resp.headers().get(&HEADER).cloned();
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let body_str = std::str::from_utf8(&body).unwrap();
        assert!(!body_str.is_empty());
        // Header and body must agree.
        assert_eq!(echoed.unwrap().to_str().unwrap(), body_str);
    }

    #[tokio::test]
    async fn propagates_incoming_header() {
        let req = Request::builder()
            .uri("/r")
            .header(HEADER.as_str(), "incoming-req-id")
            .body(Body::empty())
            .unwrap();
        let resp = router().oneshot(req).await.unwrap();
        assert_eq!(resp.headers().get(&HEADER).unwrap(), "incoming-req-id");
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(std::str::from_utf8(&body).unwrap(), "incoming-req-id");
    }
}
