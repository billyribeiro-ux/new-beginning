//! `InvoicesRepo` — append-only billing-history rows.
//!
//! BACKEND.md §4 0010. PR #8 ships the insert path; PR #10 attaches the
//! `pdf_r2_key` after the worker fetches the PDF from Stripe.

use common::ids::{InvoiceId, OrderId, UserId};
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Invoice {
    pub id: InvoiceId,
    pub order_id: Option<OrderId>,
    pub user_id: UserId,
    pub stripe_invoice_id: String,
    pub number: String,
    pub status: String,
    pub amount_cents: i64,
    pub currency: String,
    pub invoice_date: OffsetDateTime,
    pub pdf_r2_key: Option<String>,
    pub created_at: OffsetDateTime,
}

#[derive(Clone)]
pub struct InvoicesRepo {
    pool: PgPool,
}

#[derive(Debug, thiserror::Error)]
pub enum InvoicesError {
    #[error("sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
}

impl InvoicesRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Insert a new invoice or recognize the existing one (UNIQUE on
    /// `stripe_invoice_id`). Used by webhook handlers; idempotent across
    /// retries.
    #[allow(clippy::too_many_arguments)]
    pub async fn upsert_in_tx<'c>(
        &self,
        tx: &mut sqlx::Transaction<'c, sqlx::Postgres>,
        order_id: Option<OrderId>,
        user_id: UserId,
        stripe_invoice_id: &str,
        number: &str,
        status: &str,
        amount_cents: i64,
        currency: &str,
        invoice_date: OffsetDateTime,
    ) -> Result<(), InvoicesError> {
        let id = Uuid::now_v7();
        sqlx::query!(
            r#"
            INSERT INTO invoices (
                id, order_id, user_id, stripe_invoice_id, number, status,
                amount_cents, currency, invoice_date
            ) VALUES (
                $1, $2, $3, $4, $5, $6,
                $7, $8, $9
            )
            ON CONFLICT (stripe_invoice_id) DO NOTHING
            "#,
            id,
            order_id.map(|o| o.as_uuid()),
            user_id.as_uuid(),
            stripe_invoice_id,
            number,
            status,
            amount_cents,
            currency,
            invoice_date,
        )
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    /// Attach the `pdf_r2_key` after the worker fetches the Stripe PDF and
    /// puts it into R2. Idempotent: the WHERE clause permits writing the same
    /// key over an existing NULL OR matching value, but never overwrites a
    /// *different* key (which would orphan the previous object).
    pub async fn attach_pdf_r2_key(
        &self,
        stripe_invoice_id: &str,
        pdf_r2_key: &str,
    ) -> Result<u64, InvoicesError> {
        let res = sqlx::query!(
            r#"
            UPDATE invoices
            SET pdf_r2_key = $2
            WHERE stripe_invoice_id = $1
              AND (pdf_r2_key IS NULL OR pdf_r2_key = $2)
            "#,
            stripe_invoice_id,
            pdf_r2_key,
        )
        .execute(&self.pool)
        .await?;
        Ok(res.rows_affected())
    }

    /// Look up an invoice scoped to a user — the user_id predicate is the
    /// entitlement check for the `GET /v1/billing/invoices/{id}/pdf-url`
    /// route; never query by id alone from a handler.
    pub async fn find_for_user(
        &self,
        user_id: UserId,
        invoice_id: InvoiceId,
    ) -> Result<Option<Invoice>, InvoicesError> {
        let row = sqlx::query!(
            r#"
            SELECT id, order_id, user_id, stripe_invoice_id, number, status,
                   amount_cents, currency, invoice_date, pdf_r2_key, created_at
            FROM invoices
            WHERE id = $1 AND user_id = $2
            "#,
            invoice_id.as_uuid(),
            user_id.as_uuid()
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| Invoice {
            id: InvoiceId::from_uuid(r.id),
            order_id: r.order_id.map(OrderId::from_uuid),
            user_id: UserId::from_uuid(r.user_id),
            stripe_invoice_id: r.stripe_invoice_id,
            number: r.number,
            status: r.status,
            amount_cents: r.amount_cents,
            currency: r.currency,
            invoice_date: r.invoice_date,
            pdf_r2_key: r.pdf_r2_key,
            created_at: r.created_at,
        }))
    }

    /// Invoices whose `pdf_r2_key` is still NULL. Used by the worker
    /// (PR #11) to drive the Stripe PDF re-host job. `limit` caps a single
    /// sweep — the worker calls this in a loop until empty.
    pub async fn list_missing_pdf(
        &self,
        limit: i64,
    ) -> Result<Vec<(InvoiceId, UserId, String)>, InvoicesError> {
        let rows = sqlx::query!(
            r#"
            SELECT id, user_id, stripe_invoice_id
            FROM invoices
            WHERE pdf_r2_key IS NULL
            ORDER BY invoice_date DESC
            LIMIT $1
            "#,
            limit,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| {
                (
                    InvoiceId::from_uuid(r.id),
                    UserId::from_uuid(r.user_id),
                    r.stripe_invoice_id,
                )
            })
            .collect())
    }

    pub async fn list_for_user(&self, user_id: UserId) -> Result<Vec<Invoice>, InvoicesError> {
        let rows = sqlx::query!(
            r#"
            SELECT id, order_id, user_id, stripe_invoice_id, number, status,
                   amount_cents, currency, invoice_date, pdf_r2_key, created_at
            FROM invoices
            WHERE user_id = $1
            ORDER BY invoice_date DESC
            "#,
            user_id.as_uuid()
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| Invoice {
                id: InvoiceId::from_uuid(r.id),
                order_id: r.order_id.map(OrderId::from_uuid),
                user_id: UserId::from_uuid(r.user_id),
                stripe_invoice_id: r.stripe_invoice_id,
                number: r.number,
                status: r.status,
                amount_cents: r.amount_cents,
                currency: r.currency,
                invoice_date: r.invoice_date,
                pdf_r2_key: r.pdf_r2_key,
                created_at: r.created_at,
            })
            .collect())
    }
}
