//! `POST /v1/webhooks/stripe`.
//!
//! BACKEND.md §8.3 — the corrected insert-unprocessed → dispatch → mark-
//! processed flow. Crash-safe:
//!
//! 1. Signature-verify the raw body.
//! 2. Claim the event via `stripe_events.claim` (INSERT or UPDATE).
//! 3. If the row pre-existed AND already processed → 200, skip.
//! 4. Otherwise open a tx, dispatch (noop in PR #7), mark_processed_in_tx,
//!    commit. A crash anywhere in here leaves `processed_at = NULL` so
//!    PR #11's reconciliation sweep re-drives it.
//! 5. Always return 200 unless signature is invalid or body is malformed.

use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use common::alerts::AlertKind;
use stripe_client::{verify_signature, SignatureError};

use crate::state::AppState;

pub mod dispatch;

pub async fn handler(State(state): State<AppState>, headers: HeaderMap, body: Bytes) -> Response {
    // 1. Signature verify. The ONLY case where we return 4xx — Stripe will
    // back off; we want it to back off when a key rotation slips through.
    let sig_header = headers
        .get("stripe-signature")
        .and_then(|v| v.to_str().ok());
    let event = match verify_signature(&body, sig_header, &state.stripe_webhook_secret) {
        Ok(e) => e,
        Err(e) => {
            tracing::warn!(error = ?e, "stripe webhook signature failed");
            let status = match e {
                SignatureError::MissingHeader
                | SignatureError::Malformed
                | SignatureError::BadSignature
                | SignatureError::Stale
                | SignatureError::NonUtf8
                | SignatureError::BadJson(_)
                | SignatureError::KeySetup(_)
                | SignatureError::Clock => StatusCode::BAD_REQUEST,
            };
            return (status, "invalid signature").into_response();
        }
    };

    // 2. Claim (INSERT-or-recognize-existing).
    let claim = match state
        .stripe_events
        .claim(&event.id, &event.kind, &event.raw)
        .await
    {
        Ok(c) => c,
        Err(e) => {
            // Couldn't even claim — Stripe SHOULD retry (return 500-ish).
            tracing::error!(event_id = %event.id, error = ?e, "stripe_events.claim failed");
            return (StatusCode::INTERNAL_SERVER_ERROR, "claim failed").into_response();
        }
    };

    // 3. Duplicate of an already-processed event → skip.
    if !claim.freshly_inserted && claim.processed_at.is_some() {
        tracing::info!(
            event_id = %event.id,
            kind = %event.kind,
            "duplicate stripe event already processed, skipping",
        );
        return StatusCode::OK.into_response();
    }

    // 4. Open a tx, dispatch, mark_processed_in_tx, commit.
    let mut tx = match state.db.begin().await {
        Ok(t) => t,
        Err(e) => {
            tracing::error!(error = ?e, "db.begin() failed; webhook will be retried");
            return (StatusCode::INTERNAL_SERVER_ERROR, "begin tx failed").into_response();
        }
    };

    match dispatch::dispatch(&state, &mut tx, &event).await {
        Ok(()) => {
            if let Err(e) = state
                .stripe_events
                .mark_processed_in_tx(&mut tx, &event.id)
                .await
            {
                tracing::error!(event_id = %event.id, error = ?e, "mark_processed failed inside tx");
                let _ = tx.rollback().await;
                return StatusCode::OK.into_response();
            }
            if let Err(e) = tx.commit().await {
                tracing::error!(event_id = %event.id, error = ?e, "tx.commit failed");
                return StatusCode::OK.into_response();
            }
            tracing::info!(
                event_id = %event.id,
                kind = %event.kind,
                "stripe event processed",
            );
        }
        Err(e) => {
            let _ = tx.rollback().await;
            // Out-of-tx write so the error/attempts survive the rollback.
            let _ = state
                .stripe_events
                .mark_failed(&event.id, &e.to_string())
                .await;
            tracing::error!(event_id = %event.id, error = ?e, "stripe handler failed");
            state.alerts.fire_async(AlertKind::WebhookHandlerFailed {
                event_id: event.id.clone(),
                error: e.to_string(),
            });
            // Still 200 — we already deduped. The reconciliation cron will
            // re-drive on the next sweep.
        }
    }

    StatusCode::OK.into_response()
}

// `dispatch` body lives in `dispatch.rs` — same module path so the
// import surface from `mod webhooks::stripe` stays `dispatch::dispatch`.
