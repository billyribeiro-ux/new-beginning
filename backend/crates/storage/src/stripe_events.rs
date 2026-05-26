//! `StripeEventsRepo` — webhook idempotency + lifecycle.
//!
//! BACKEND.md §8.3. The corrected claim query handles the crash-mid-dispatch
//! case explicitly: a row already present with `processed_at IS NULL` means
//! a previous attempt died after the INSERT but before the dispatch
//! committed. We let the new attempt re-drive the dispatcher.
//!
//! ```sql
//! INSERT INTO stripe_events (event_id, event_type, payload, received_at)
//! VALUES ($1, $2, $3, now())
//! ON CONFLICT (event_id) DO UPDATE SET event_id = stripe_events.event_id
//! RETURNING (xmax = 0) AS freshly_inserted, processed_at
//! ```
//!
//! The `xmax = 0` trick is Postgres-specific: `xmax` is zero only on a row
//! that was actually inserted, non-zero on an updated row.

use sqlx::PgPool;
use time::OffsetDateTime;

#[derive(Debug, Clone)]
pub struct EventClaim {
    pub event_id: String,
    pub freshly_inserted: bool,
    pub processed_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone)]
pub struct StoredEvent {
    pub event_id: String,
    pub event_type: String,
    pub received_at: OffsetDateTime,
    pub processed_at: Option<OffsetDateTime>,
    pub attempts: i32,
    pub processing_error: Option<String>,
}

#[derive(Clone)]
pub struct StripeEventsRepo {
    pool: PgPool,
}

#[derive(Debug, thiserror::Error)]
pub enum StripeEventsError {
    #[error("sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
}

impl StripeEventsRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Insert (or re-claim) the event. The caller MUST inspect
    /// `freshly_inserted` + `processed_at` to decide whether to dispatch
    /// (BACKEND.md §8.3 rules 3 + 4).
    pub async fn claim(
        &self,
        event_id: &str,
        event_type: &str,
        payload: &serde_json::Value,
    ) -> Result<EventClaim, StripeEventsError> {
        let row = sqlx::query!(
            r#"
            INSERT INTO stripe_events (event_id, event_type, payload, received_at)
            VALUES ($1, $2, $3, now())
            ON CONFLICT (event_id) DO UPDATE SET event_id = stripe_events.event_id
            RETURNING (xmax = 0) AS "freshly_inserted!", processed_at
            "#,
            event_id,
            event_type,
            payload,
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(EventClaim {
            event_id: event_id.to_string(),
            freshly_inserted: row.freshly_inserted,
            processed_at: row.processed_at,
        })
    }

    /// Stamp `processed_at = now()` and bump `attempts`. Must run inside
    /// the SAME transaction that committed the side effects so a crash
    /// after dispatch but before this update leaves the row re-drivable
    /// (BACKEND.md §8.3).
    pub async fn mark_processed_in_tx<'c>(
        &self,
        tx: &mut sqlx::Transaction<'c, sqlx::Postgres>,
        event_id: &str,
    ) -> Result<(), StripeEventsError> {
        sqlx::query!(
            r#"
            UPDATE stripe_events
            SET processed_at = now(), attempts = attempts + 1, processing_error = NULL
            WHERE event_id = $1
            "#,
            event_id,
        )
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    /// On dispatcher error: bump `attempts` + record the message, leave
    /// `processed_at` NULL. NOT in a tx — we want this write to survive
    /// the rollback of the failed dispatch.
    pub async fn mark_failed(&self, event_id: &str, error: &str) -> Result<(), StripeEventsError> {
        sqlx::query!(
            r#"
            UPDATE stripe_events
            SET attempts = attempts + 1, processing_error = $2
            WHERE event_id = $1
            "#,
            event_id,
            error,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Read a single row. Used by tests + by PR #11's reconciliation cron.
    pub async fn find(&self, event_id: &str) -> Result<Option<StoredEvent>, StripeEventsError> {
        let row = sqlx::query!(
            r#"
            SELECT event_id, event_type, received_at, processed_at, attempts, processing_error
            FROM stripe_events
            WHERE event_id = $1
            "#,
            event_id,
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| StoredEvent {
            event_id: r.event_id,
            event_type: r.event_type,
            received_at: r.received_at,
            processed_at: r.processed_at,
            attempts: r.attempts,
            processing_error: r.processing_error,
        }))
    }

    /// Events received but never processed, older than `older_than_seconds`.
    /// Drives PR #11's `reconcile_stripe_events` cron — the load-bearing
    /// backstop against a crashed dispatch.
    pub async fn list_unprocessed_older_than(
        &self,
        older_than_seconds: i64,
        limit: i64,
    ) -> Result<Vec<StoredEvent>, StripeEventsError> {
        let rows = sqlx::query!(
            r#"
            SELECT event_id, event_type, received_at, processed_at, attempts, processing_error
            FROM stripe_events
            WHERE processed_at IS NULL
              AND received_at < now() - make_interval(secs => $1)
            ORDER BY received_at
            LIMIT $2
            "#,
            older_than_seconds as f64,
            limit,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| StoredEvent {
                event_id: r.event_id,
                event_type: r.event_type,
                received_at: r.received_at,
                processed_at: r.processed_at,
                attempts: r.attempts,
                processing_error: r.processing_error,
            })
            .collect())
    }
}
