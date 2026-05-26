//! `GET /v1/billing/invoices` and `GET /v1/billing/invoices/{id}/pdf-url`.
//!
//! BACKEND.md §8.7. Two halves:
//!   * `list`: the billing-history table — every invoice the user has,
//!     paid or not, newest first. The row's `pdf_r2_key` may be null
//!     (worker hasn't fetched it yet); the UI hides the download link in
//!     that case.
//!   * `pdf_url`: mints a short-TTL presigned GET against R2. The
//!     entitlement check is the `user_id` predicate in
//!     `InvoicesRepo::find_for_user` — never look up by id alone.

use axum::extract::{Path, State};
use axum::Json;
use common::error::AppError;
use common::ids::InvoiceId;
use serde::Serialize;
use std::time::Duration;
use time::OffsetDateTime;

use crate::auth::AuthSession;
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct InvoiceRow {
    pub id: InvoiceId,
    pub number: String,
    pub status: String,
    pub amount_cents: i64,
    pub currency: String,
    #[serde(with = "time::serde::rfc3339")]
    pub invoice_date: OffsetDateTime,
    pub has_pdf: bool,
}

#[derive(Debug, Serialize)]
pub struct ListResponse {
    pub invoices: Vec<InvoiceRow>,
}

pub async fn list(
    State(state): State<AppState>,
    session: AuthSession,
) -> Result<Json<ListResponse>, AppError> {
    let invoices = state
        .invoices
        .list_for_user(session.user_id)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    let invoices = invoices
        .into_iter()
        .map(|inv| InvoiceRow {
            id: inv.id,
            number: inv.number,
            status: inv.status,
            amount_cents: inv.amount_cents,
            currency: inv.currency,
            invoice_date: inv.invoice_date,
            has_pdf: inv.pdf_r2_key.is_some(),
        })
        .collect();
    Ok(Json(ListResponse { invoices }))
}

#[derive(Debug, Serialize)]
pub struct PdfUrlResponse {
    pub url: String,
    /// TTL in seconds, mirrored for the BFF so it can cache-control
    /// accordingly. Always equals `PDF_URL_TTL`.
    pub ttl_seconds: u64,
}

/// Short TTL — the user clicks "download", hits this endpoint, gets a URL,
/// and follows it immediately. Longer TTL would let leaked URLs sit usable
/// for longer with no upside. BACKEND.md §9.
const PDF_URL_TTL: Duration = Duration::from_secs(300);

pub async fn pdf_url(
    State(state): State<AppState>,
    session: AuthSession,
    Path(invoice_id): Path<InvoiceId>,
) -> Result<Json<PdfUrlResponse>, AppError> {
    let invoice = state
        .invoices
        .find_for_user(session.user_id, invoice_id)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?
        .ok_or(AppError::NotFound)?;

    let key = invoice.pdf_r2_key.ok_or(AppError::NotFound)?;

    // HEAD before sign — catches the "DB row says we have it, but R2 lost
    // it" case so we 404 instead of handing the user a URL that 403s
    // mid-download.
    state.r2.head_object(&key).await.map_err(|e| match e {
        r2_client::StoreError::NotFound => AppError::NotFound,
        other => AppError::External {
            service: "r2",
            source: anyhow::anyhow!(other),
        },
    })?;

    let url = state
        .r2
        .presigned_get(&key, PDF_URL_TTL)
        .map_err(|e| AppError::External {
            service: "r2",
            source: anyhow::anyhow!(e),
        })?;

    Ok(Json(PdfUrlResponse {
        url: url.to_string(),
        ttl_seconds: PDF_URL_TTL.as_secs(),
    }))
}
