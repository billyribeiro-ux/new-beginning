//! `reconcile_stripe_events` — the backstop for crashed mid-dispatch
//! webhooks.
//!
//! BACKEND.md §8.3 (5-min reconciliation): re-drives every `stripe_events`
//! row with `processed_at IS NULL` that's older than `min_age_secs`. Uses
//! the EXACT same `dispatch` fn as the webhook receiver — load-bearing
//! equivalence per BACKEND.md §22 rule 4.
//!
//! Returns the number of rows successfully re-driven. Errors per row are
//! logged + the row stays `processed_at IS NULL` for the next sweep.

use stripe_client::ParsedEvent;
use tracing::instrument;

use crate::routes::webhooks::stripe::dispatch::dispatch;
use crate::state::AppState;

/// One sweep. PR #11's worker calls this on a 5-min cron; PR #8 ships it
/// as a standalone fn that's also exercised by the evidence script.
///
/// Re-uses the receiver's `claim → dispatch → mark_processed_in_tx` flow
/// MINUS the signature check (the row already passed signature on its
/// first delivery).
#[instrument(skip(state), fields(min_age_secs))]
pub async fn reconcile_stripe_events(
    state: &AppState,
    min_age_secs: i64,
    limit: i64,
) -> anyhow::Result<u64> {
    let rows = state
        .stripe_events
        .list_unprocessed_older_than(min_age_secs, limit)
        .await?;
    if rows.is_empty() {
        return Ok(0);
    }
    tracing::info!(
        candidates = rows.len(),
        min_age_secs,
        "reconciliation sweep starting",
    );

    let mut redriven = 0u64;
    for row in rows {
        // Re-fetch the payload so we can reconstruct a `ParsedEvent`.
        // BACKEND.md §11 says the production cron re-fetches the event
        // from Stripe (`GET /v1/events/{id}`) so the payload is fresh
        // even if we no longer have it locally. PR #8 reads it back from
        // our own `stripe_events.payload` column — works for the
        // crash-recovery scenario (payload was written before the
        // crash). PR #11's cron adds the GET fallback.
        let raw = match fetch_payload(state, &row.event_id).await {
            Some(r) => r,
            None => {
                tracing::warn!(
                    event_id = %row.event_id,
                    "reconcile: payload missing locally; skipping (PR #11 adds Stripe re-fetch)",
                );
                continue;
            }
        };
        let event = ParsedEvent {
            id: row.event_id.clone(),
            kind: row.event_type.clone(),
            raw,
        };

        let mut tx = state.db.begin().await?;
        match dispatch(state, &mut tx, &event).await {
            Ok(()) => {
                if let Err(e) = state
                    .stripe_events
                    .mark_processed_in_tx(&mut tx, &event.id)
                    .await
                {
                    tracing::error!(event_id = %event.id, error = ?e, "reconcile: mark_processed failed");
                    let _ = tx.rollback().await;
                    continue;
                }
                if let Err(e) = tx.commit().await {
                    tracing::error!(event_id = %event.id, error = ?e, "reconcile: commit failed");
                    continue;
                }
                redriven += 1;
                tracing::info!(event_id = %event.id, "reconcile: re-drove event");
            }
            Err(e) => {
                let _ = tx.rollback().await;
                let _ = state
                    .stripe_events
                    .mark_failed(&event.id, &e.to_string())
                    .await;
                tracing::error!(event_id = %event.id, error = ?e, "reconcile: dispatch failed");
            }
        }
    }
    Ok(redriven)
}

async fn fetch_payload(state: &AppState, event_id: &str) -> Option<serde_json::Value> {
    sqlx::query!(
        "SELECT payload FROM stripe_events WHERE event_id = $1",
        event_id
    )
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten()
    .and_then(|r| r.payload)
}
