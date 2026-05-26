//! `AppError` — the single error type that every HTTP handler returns.
//!
//! BACKEND.md §14: maps to an HTTP status and a `{ error: { code, message },
//! request_id }` JSON body. `Internal` / `External` never echo the source
//! string to the client; they log it with the request id and (per §6) may fire
//! an `AlertSink` event.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::money::MoneyError;
use crate::request_id::current_request_id;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("validation: {0}")]
    Validation(String),

    #[error("unauthorized")]
    Unauthorized,

    #[error("forbidden")]
    Forbidden,

    #[error("not found")]
    NotFound,

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("rate limited")]
    RateLimited { retry_after_secs: u32 },

    #[error("external: {service}")]
    External {
        service: &'static str,
        #[source]
        source: anyhow::Error,
    },

    #[error("internal")]
    Internal(#[from] anyhow::Error),
}

impl AppError {
    fn status_and_code(&self) -> (StatusCode, &'static str) {
        match self {
            AppError::Validation(_) => (StatusCode::UNPROCESSABLE_ENTITY, "validation"),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized"),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "forbidden"),
            AppError::NotFound => (StatusCode::NOT_FOUND, "not_found"),
            AppError::Conflict(_) => (StatusCode::CONFLICT, "conflict"),
            AppError::RateLimited { .. } => (StatusCode::TOO_MANY_REQUESTS, "rate_limited"),
            AppError::External { .. } => (StatusCode::BAD_GATEWAY, "upstream_error"),
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal"),
        }
    }

    /// Public-safe message. `Internal` / `External` never leak the source string.
    fn public_message(&self) -> String {
        match self {
            AppError::Validation(m) | AppError::Conflict(m) => m.clone(),
            AppError::Unauthorized => "authentication required".into(),
            AppError::Forbidden => "forbidden".into(),
            AppError::NotFound => "not found".into(),
            AppError::RateLimited { .. } => "rate limited".into(),
            AppError::External { service, .. } => {
                format!("upstream service unavailable: {service}")
            }
            AppError::Internal(_) => "internal error".into(),
        }
    }
}

/// `MoneyError` is always a bug (BACKEND.md §14): surface loudly as Internal.
impl From<MoneyError> for AppError {
    fn from(e: MoneyError) -> Self {
        AppError::Internal(anyhow::anyhow!(e))
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let request_id = current_request_id().unwrap_or_default();
        let (status, code) = self.status_and_code();

        // Log internals at the appropriate level; never echo them to the body.
        match &self {
            AppError::Internal(e) => {
                tracing::error!(error = ?e, request_id = %request_id, "internal error");
            }
            AppError::External { service, source } => {
                tracing::error!(service = %service, error = ?source, request_id = %request_id, "external error");
            }
            AppError::Conflict(m) | AppError::Validation(m) => {
                tracing::info!(message = %m, request_id = %request_id, kind = %code, "user error");
            }
            _ => {
                tracing::debug!(request_id = %request_id, kind = %code, "client error");
            }
        }

        let body = Json(json!({
            "error": { "code": code, "message": self.public_message() },
            "request_id": request_id,
        }));

        let mut resp = (status, body).into_response();
        if let AppError::RateLimited { retry_after_secs } = self {
            if let Ok(v) = http::HeaderValue::from_str(&retry_after_secs.to_string()) {
                resp.headers_mut().insert(http::header::RETRY_AFTER, v);
            }
        }
        resp
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;

    #[tokio::test]
    async fn internal_error_does_not_leak_source() {
        let err = AppError::Internal(anyhow::anyhow!(
            "DB connection string contains password=hunter2"
        ));
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let body = to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
        let s = std::str::from_utf8(&body).unwrap();
        assert!(!s.contains("hunter2"), "leaked internal source: {s}");
        assert!(s.contains("internal error"));
    }

    #[tokio::test]
    async fn rate_limited_attaches_retry_after() {
        let err = AppError::RateLimited {
            retry_after_secs: 42,
        };
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(resp.headers().get(http::header::RETRY_AFTER).unwrap(), "42");
    }

    #[tokio::test]
    async fn validation_passes_through_user_message() {
        let err = AppError::Validation("email must be present".into());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let body = to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
        let s = std::str::from_utf8(&body).unwrap();
        assert!(s.contains("email must be present"));
    }

    #[tokio::test]
    async fn money_error_converts_to_internal() {
        let err: AppError = MoneyError::Overflow.into();
        match err {
            AppError::Internal(_) => {}
            other => panic!("expected Internal, got {other:?}"),
        }
    }
}
