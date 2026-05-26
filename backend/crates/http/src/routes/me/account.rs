//! `GET /v1/me/export` and `DELETE /v1/me` — data export + account deletion.
//!
//! BACKEND.md §16 (PR #16).
//!
//! * **Export**: inline JSON dump of every row a user owns. Sufficient
//!   for the typical "give me my data" GDPR request. A future
//!   background job writes a gzipped JSON to R2 + emails a presigned
//!   URL; for today the inline response is enough (TradeFlex users
//!   have small data footprints).
//! * **Delete**: soft-delete via `UsersRepo::soft_delete` — stamps
//!   `deleted_at`, pseudonymizes the email, clears creds. Sessions are
//!   purged in the same tx so a stolen cookie can't outlive the
//!   deletion. Orders / invoices / audit are retained per BACKEND.md
//!   §16 (legal/accounting); a follow-up retention job hard-purges
//!   after the retention window.

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use common::error::AppError;
use serde::Serialize;
use serde_json::Value;
use time::OffsetDateTime;

use crate::auth::AuthSession;
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct ExportResponse {
    pub user: Value,
    pub orders: Value,
    pub invoices: Value,
    pub licenses: Value,
    pub enrollments: Value,
    pub subscriptions: Value,
    pub notifications: Value,
    #[serde(with = "time::serde::rfc3339")]
    pub generated_at: OffsetDateTime,
}

pub async fn export(
    State(state): State<AppState>,
    session: AuthSession,
) -> Result<Json<ExportResponse>, AppError> {
    // Each block: serde_json::Value-wrap the rows for a self-contained
    // payload. The handler does NOT touch sensitive columns (password
    // hashes, encrypted TOTP secret, raw IP) — only what the user can
    // already see in the UI.
    let user_row = sqlx::query!(
        r#"
        SELECT id, email::text AS "email!", name, headline, role, created_at
        FROM users WHERE id = $1
        "#,
        session.user_id.as_uuid()
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    let user = serde_json::json!({
        "id": user_row.id,
        "email": user_row.email,
        "name": user_row.name,
        "headline": user_row.headline,
        "role": user_row.role,
        "created_at": user_row.created_at,
    });

    let orders_raw = sqlx::query!(
        r#"
        SELECT id, status, subtotal_cents, tax_cents, total_cents,
               currency, created_at, paid_at, refunded_at
        FROM orders WHERE user_id = $1 ORDER BY created_at DESC
        "#,
        session.user_id.as_uuid()
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    let orders = serde_json::to_value(
        orders_raw
            .into_iter()
            .map(|r| {
                serde_json::json!({
                    "id": r.id,
                    "status": r.status,
                    "subtotal_cents": r.subtotal_cents,
                    "tax_cents": r.tax_cents,
                    "total_cents": r.total_cents,
                    "currency": r.currency,
                    "created_at": r.created_at,
                    "paid_at": r.paid_at,
                    "refunded_at": r.refunded_at,
                })
            })
            .collect::<Vec<_>>(),
    )
    .unwrap();

    let invoices_raw = state
        .invoices
        .list_for_user(session.user_id)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    let invoices = serde_json::to_value(
        invoices_raw
            .into_iter()
            .map(|i| {
                serde_json::json!({
                    "id": i.id,
                    "number": i.number,
                    "status": i.status,
                    "amount_cents": i.amount_cents,
                    "currency": i.currency,
                    "invoice_date": i.invoice_date,
                    "has_pdf": i.pdf_r2_key.is_some(),
                })
            })
            .collect::<Vec<_>>(),
    )
    .unwrap();

    let licenses_raw = state
        .licenses
        .list_active_for_user(session.user_id)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    let licenses = serde_json::to_value(
        licenses_raw
            .into_iter()
            .map(|l| {
                serde_json::json!({
                    "id": l.id,
                    "product_id": l.product_id,
                    "license_key_prefix": l.license_key_prefix,
                    "source": l.source,
                    "issued_at": l.issued_at,
                })
            })
            .collect::<Vec<_>>(),
    )
    .unwrap();

    let enrollments_raw = state
        .enrollments
        .list_for_user(session.user_id)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    let enrollments = serde_json::to_value(
        enrollments_raw
            .into_iter()
            .map(|e| {
                serde_json::json!({
                    "id": e.id,
                    "product_id": e.product_id,
                    "source": e.source,
                    "active": e.active,
                    "started_at": e.started_at,
                })
            })
            .collect::<Vec<_>>(),
    )
    .unwrap();

    let subscriptions = match state
        .subscriptions
        .find_active_for_user(session.user_id)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?
    {
        Some(s) => serde_json::json!([{
            "id": s.id,
            "plan_id": s.plan_id,
            "status": s.status,
            "current_period_start": s.current_period_start,
            "current_period_end": s.current_period_end,
            "cancel_at_period_end": s.cancel_at_period_end,
            "canceled_at": s.canceled_at,
        }]),
        None => serde_json::json!([]),
    };

    let notifications_raw = state
        .notifications
        .list_for_user(session.user_id, false, 1000)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    let notifications = serde_json::to_value(
        notifications_raw
            .into_iter()
            .map(|n| {
                serde_json::json!({
                    "id": n.id,
                    "kind": n.kind,
                    "title": n.title,
                    "body": n.body,
                    "source": n.source,
                    "read_at": n.read_at,
                    "created_at": n.created_at,
                })
            })
            .collect::<Vec<_>>(),
    )
    .unwrap();

    Ok(Json(ExportResponse {
        user,
        orders,
        invoices,
        licenses,
        enrollments,
        subscriptions,
        notifications,
        generated_at: OffsetDateTime::now_utc(),
    }))
}

pub async fn delete_me(
    State(state): State<AppState>,
    session: AuthSession,
) -> Result<StatusCode, AppError> {
    let n = state
        .users
        .soft_delete(session.user_id)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    if n == 0 {
        // Already deleted — idempotent.
        return Ok(StatusCode::NO_CONTENT);
    }
    // Audit (after the soft-delete commit). Records the actor + target.
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
            "account.deleted",
            "users",
            &session.user_id.to_string(),
            serde_json::json!({}),
            None,
        )
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    tx.commit()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    Ok(StatusCode::NO_CONTENT)
}
