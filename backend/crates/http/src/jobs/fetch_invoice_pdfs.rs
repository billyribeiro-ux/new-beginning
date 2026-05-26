//! `fetch_invoice_pdfs` — worker job that re-hosts Stripe invoice PDFs
//! into R2.
//!
//! BACKEND.md §21 PR #10 + #11. Stripe hosts each invoice's PDF behind a
//! short-TTL URL that rotates; rather than persisting that URL we re-fetch
//! the invoice from Stripe each time we plan to download. The fetched
//! bytes go into R2 at the canonical key
//! `invoices/{user_id}/{invoice_id}.pdf`, and the row's `pdf_r2_key`
//! column is set so [`InvoicesRepo::find_for_user`] reports it as
//! downloadable.
//!
//! The job is **idempotent**: it skips rows already keyed, and the
//! [`InvoicesRepo::attach_pdf_r2_key`] update is itself idempotent on the
//! same key. Re-running is safe.

use bytes::Bytes;
use r2_client::keys;
use tracing::instrument;

use crate::state::AppState;

/// Per-row result, for evidence/logging. The caller drops these into
/// metrics if needed.
#[derive(Debug, Clone, Copy)]
pub enum InvoicePdfOutcome {
    Uploaded,
    NoStripePdfUrl,
    UpstreamError,
    SkippedAlreadyAttached,
}

/// One sweep. Lists up to `limit` invoices with `pdf_r2_key IS NULL`, then
/// for each: GET the Stripe invoice → GET the PDF bytes → PUT to R2 →
/// UPDATE `invoices.pdf_r2_key`. Errors are logged per row; the row stays
/// NULL for the next sweep.
#[instrument(skip(state), fields(limit))]
pub async fn fetch_invoice_pdfs(state: &AppState, limit: i64) -> anyhow::Result<u64> {
    let rows = state.invoices.list_missing_pdf(limit).await?;
    if rows.is_empty() {
        return Ok(0);
    }
    tracing::info!(candidates = rows.len(), "invoice-pdf sweep starting");

    // Dedicated HTTP client for downloading PDF bytes from Stripe's hosted
    // URL. We can't reuse the StripeClient's reqwest because that one is
    // pinned to api.stripe.com with our bearer key; the invoice_pdf URL is
    // a different host (files.stripe.com / stripe.com) and uses a query-
    // string token instead of bearer auth.
    let pdf_http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .connect_timeout(std::time::Duration::from_secs(5))
        .build()?;

    let mut uploaded = 0u64;
    for (invoice_id, user_id, stripe_invoice_id) in rows {
        match fetch_one(state, &pdf_http, invoice_id, user_id, &stripe_invoice_id).await {
            Ok(InvoicePdfOutcome::Uploaded) => {
                uploaded += 1;
                tracing::info!(
                    %invoice_id, %stripe_invoice_id,
                    "invoice-pdf: re-hosted to R2",
                );
            }
            Ok(other) => {
                tracing::debug!(
                    %invoice_id, %stripe_invoice_id, outcome = ?other,
                    "invoice-pdf: skipped",
                );
            }
            Err(e) => {
                tracing::error!(
                    %invoice_id, %stripe_invoice_id, error = ?e,
                    "invoice-pdf: error (row stays NULL for next sweep)",
                );
            }
        }
    }
    Ok(uploaded)
}

async fn fetch_one(
    state: &AppState,
    pdf_http: &reqwest::Client,
    invoice_id: common::ids::InvoiceId,
    user_id: common::ids::UserId,
    stripe_invoice_id: &str,
) -> anyhow::Result<InvoicePdfOutcome> {
    let inv = state.stripe.get_invoice(stripe_invoice_id).await?;
    let Some(pdf_url) = inv.invoice_pdf else {
        return Ok(InvoicePdfOutcome::NoStripePdfUrl);
    };

    // Download the bytes. Stripe's invoice PDFs are typically <500 KB.
    let resp = pdf_http.get(&pdf_url).send().await?;
    if !resp.status().is_success() {
        tracing::warn!(
            status = %resp.status(),
            stripe_invoice_id,
            "invoice-pdf: upstream non-2xx",
        );
        return Ok(InvoicePdfOutcome::UpstreamError);
    }
    let bytes: Bytes = resp.bytes().await?;

    let key = keys::invoice_pdf(user_id, invoice_id);
    state.r2.put_object(&key, bytes, "application/pdf").await?;

    let updated = state
        .invoices
        .attach_pdf_r2_key(stripe_invoice_id, &key)
        .await?;
    if updated == 0 {
        // Someone else won the race (or wrote a different key); the
        // upload we just did is now orphaned. Log so we can clean up.
        tracing::warn!(
            stripe_invoice_id,
            r2_key = %key,
            "invoice-pdf: attach updated 0 rows — possible race or pre-existing different key",
        );
        return Ok(InvoicePdfOutcome::SkippedAlreadyAttached);
    }
    Ok(InvoicePdfOutcome::Uploaded)
}
