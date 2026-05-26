//! `POST /v1/admin/orders/{id}/refund` — admin-triggered Stripe refund.
//!
//! BACKEND.md §15 (refund flow). The admin clicks "refund"; we call
//! Stripe with an Idempotency-Key so a double-click can't double-refund;
//! Stripe POSTs back `charge.refunded` to the webhook receiver, which
//! flips the order to `'refunded'` + revokes entitlements + drops a
//! notification.
//!
//! The api binary deliberately does NOT revoke entitlements inline at
//! refund-trigger time — keeping the source of truth single. If the
//! Stripe call returns 200 but the webhook never arrives (network
//! partition), the reconciliation cron (`reconcile_stripe_events`) will
//! re-fetch the event from Stripe and dispatch it.

use axum::extract::{Path, State};
use axum::Json;
use common::auth::idempotency;
use common::error::AppError;
use common::ids::OrderId;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::auth::AuthSession;
use crate::state::AppState;

#[derive(Debug, Deserialize, Validate)]
pub struct RefundRequest {
    /// Free-form reason — surfaced to the audit row. Required so
    /// "why was this refunded?" is always answerable from `audit_log`.
    #[validate(length(min = 4, max = 500))]
    pub reason: String,
}

#[derive(Debug, Serialize)]
pub struct RefundResponse {
    pub stripe_refund_id: String,
    pub status: Option<String>,
}

pub async fn trigger(
    State(state): State<AppState>,
    session: AuthSession,
    Path(order_id): Path<OrderId>,
    Json(req): Json<RefundRequest>,
) -> Result<Json<RefundResponse>, AppError> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let order = state
        .orders
        .find_by_id(order_id)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?
        .ok_or(AppError::NotFound)?;

    if order.status != "paid" {
        return Err(AppError::Conflict(format!(
            "order {order_id} is in status '{}' — can only refund 'paid' orders",
            order.status
        )));
    }

    // Resolve the payment intent. The schema column might be missing if
    // the webhook never landed (extremely unusual since we wouldn't be
    // in `status='paid'` then). 422 with a clear message in that case.
    let pi: Option<String> = sqlx::query!(
        "SELECT stripe_payment_intent_id FROM orders WHERE id = $1",
        order_id.as_uuid()
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?
    .stripe_payment_intent_id;

    let payment_intent_id = pi.ok_or_else(|| {
        AppError::Validation(
            "order has no recorded payment_intent — cannot refund automatically; refund via Stripe Dashboard"
                .into(),
        )
    })?;

    // Idempotency-Key: (env, actor, "order_refund", order_id). A
    // double-click from the same admin against the same order resolves
    // to the same key → Stripe returns the original refund.
    let idem = idempotency::derive_key(
        &state.config.env,
        session.user_id,
        "order_refund",
        &order_id.to_string(),
    );

    let refund = state
        .stripe
        .refund_payment_intent(&payment_intent_id, &idem)
        .await
        .map_err(|e| AppError::External {
            service: "stripe",
            source: anyhow::anyhow!(e),
        })?;

    // Audit the TRIGGER (separate from the `order.refunded` audit row
    // the webhook handler writes on success — that's the state-mutation
    // record; this one records the human intent). Note: this audit row
    // is written outside any tx because it's a leaf write; the
    // `charge.refunded` handler will write the in-tx record when Stripe
    // confirms.
    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    state
        .audit
        .record_in_tx(
            &mut tx,
            Some(session.user_id),
            "admin.refund.triggered",
            "orders",
            &order_id.to_string(),
            serde_json::json!({
                "payment_intent_id": payment_intent_id,
                "stripe_refund_id": refund.id,
                "reason": req.reason,
            }),
            None,
        )
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    tx.commit()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

    Ok(Json(RefundResponse {
        stripe_refund_id: refund.id,
        status: refund.status,
    }))
}
