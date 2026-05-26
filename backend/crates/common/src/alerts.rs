//! `AlertSink` — the one canonical mechanism for "should-never-happen" reporting.
//!
//! BACKEND.md §6 + §14: the webhook handler and `AppError::into_response` both
//! fire alerts through this trait. The prod impl pages an on-call channel
//! (Resend/Slack/etc.); tests use the `RecordingSink` below to assert on
//! emitted events.
//!
//! Fire-and-forget: an alert that can fail isn't an alert. The trait spawns
//! its own task and never returns an error to the caller.

use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlertKind {
    /// A Stripe webhook event passed signature verification but the handler
    /// failed mid-dispatch. The reconciliation sweep will re-drive it.
    WebhookHandlerFailed { event_id: String, error: String },

    /// A `background_jobs` row exhausted retries.
    JobDeadLettered {
        kind: String,
        job_id: String,
        error: String,
    },

    /// `/readyz` flipped a check to failing.
    ReadinessDegraded { check: &'static str },
}

pub trait AlertSink: Send + Sync {
    /// Fire-and-forget. Implementations spawn their own task and never block
    /// the caller. There is no return value because there is no recovery
    /// action a caller could take if alerting itself failed.
    fn fire_async(&self, kind: AlertKind);
}

/// Logs alerts via `tracing` at error level. Useful as a default in local dev
/// and as a fallback when the real pager is unavailable.
#[derive(Debug, Default, Clone)]
pub struct LogAlertSink;

impl AlertSink for LogAlertSink {
    fn fire_async(&self, kind: AlertKind) {
        // No spawn needed for a pure tracing emit.
        match kind {
            AlertKind::WebhookHandlerFailed { event_id, error } => {
                tracing::error!(event_id = %event_id, error = %error, "ALERT: webhook handler failed");
            }
            AlertKind::JobDeadLettered {
                kind,
                job_id,
                error,
            } => {
                tracing::error!(kind = %kind, job_id = %job_id, error = %error, "ALERT: job dead-lettered");
            }
            AlertKind::ReadinessDegraded { check } => {
                tracing::error!(check = %check, "ALERT: readiness degraded");
            }
        }
    }
}

/// Test-only sink that captures every fired alert into an in-memory vector.
#[derive(Debug, Default, Clone)]
pub struct RecordingSink {
    inner: Arc<Mutex<Vec<AlertKind>>>,
}

impl RecordingSink {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn recorded(&self) -> Vec<AlertKind> {
        self.inner.lock().expect("alerts mutex poisoned").clone()
    }
}

impl AlertSink for RecordingSink {
    fn fire_async(&self, kind: AlertKind) {
        self.inner.lock().expect("alerts mutex poisoned").push(kind);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recording_sink_captures_alerts() {
        let sink = RecordingSink::new();
        sink.fire_async(AlertKind::ReadinessDegraded { check: "db" });
        sink.fire_async(AlertKind::JobDeadLettered {
            kind: "send_email".into(),
            job_id: "abc".into(),
            error: "smtp 502".into(),
        });

        let got = sink.recorded();
        assert_eq!(got.len(), 2);
        assert_eq!(got[0], AlertKind::ReadinessDegraded { check: "db" });
    }

    #[test]
    fn log_alert_sink_does_not_panic() {
        let sink = LogAlertSink;
        sink.fire_async(AlertKind::WebhookHandlerFailed {
            event_id: "evt_test".into(),
            error: "boom".into(),
        });
    }
}
