//! Request-id propagation via `tokio::task_local`.
//!
//! BACKEND.md §15: middleware installs the incoming `x-request-id` (or
//! generates a fresh UUIDv7) into a task-local before each handler runs.
//! `AppError::into_response` reads it back so error JSON always carries the
//! id, even when the handler returned `Err` early and never wrote a span.

tokio::task_local! {
    static REQUEST_ID: String;
}

/// Run `f` with `request_id` set as the current task's request id.
pub async fn with_request_id<F, R>(request_id: String, f: F) -> R
where
    F: std::future::Future<Output = R>,
{
    REQUEST_ID.scope(request_id, f).await
}

/// Read the current task's request id, if any.
///
/// Returns `None` outside of a `with_request_id` scope (which is intentional —
/// background jobs and tests run without a request id and the error path must
/// still produce a body).
pub fn current_request_id() -> Option<String> {
    REQUEST_ID.try_with(|id| id.clone()).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn current_returns_none_outside_scope() {
        assert!(current_request_id().is_none());
    }

    #[tokio::test]
    async fn with_request_id_makes_it_readable() {
        let got = with_request_id("req-123".into(), async { current_request_id() }).await;
        assert_eq!(got.as_deref(), Some("req-123"));
    }

    #[tokio::test]
    async fn scopes_do_not_leak() {
        let inside = with_request_id("inner".into(), async { current_request_id() }).await;
        assert_eq!(inside.as_deref(), Some("inner"));
        assert!(current_request_id().is_none());
    }
}
