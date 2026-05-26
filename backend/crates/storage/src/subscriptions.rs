//! `SubscriptionsRepo` — local mirror of Stripe subscription state.
//!
//! BACKEND.md §1.3: Stripe is the source of truth; this mirror exists so
//! dashboard / entitlement reads don't pay Stripe-API latency. Webhooks
//! keep it in sync (`PR #8` does first-create; PR #9 keeps it fresh).

use common::ids::{PlanId, SubscriptionId, UserId};
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Subscription {
    pub id: SubscriptionId,
    pub user_id: UserId,
    pub plan_id: PlanId,
    pub stripe_subscription_id: String,
    pub status: String,
    pub cancel_at_period_end: bool,
    pub current_period_start: OffsetDateTime,
    pub current_period_end: OffsetDateTime,
    pub canceled_at: Option<OffsetDateTime>,
}

#[derive(Clone)]
pub struct SubscriptionsRepo {
    pool: PgPool,
}

#[derive(Debug, thiserror::Error)]
pub enum SubscriptionsError {
    #[error("sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
}

impl SubscriptionsRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// First-create / upsert from a Stripe webhook. The `(stripe_subscription_id)`
    /// UNIQUE constraint makes the conflict path collapse to an UPDATE
    /// that refreshes status + period markers (PR #9 wires the periodic
    /// `customer.subscription.updated` path; PR #8 only ever hits this
    /// from `checkout.session.completed` when `mode = subscription`).
    #[allow(clippy::too_many_arguments)]
    pub async fn upsert_from_stripe_in_tx<'c>(
        &self,
        tx: &mut sqlx::Transaction<'c, sqlx::Postgres>,
        user_id: UserId,
        plan_id: PlanId,
        stripe_subscription_id: &str,
        status: &str,
        cancel_at_period_end: bool,
        current_period_start: OffsetDateTime,
        current_period_end: OffsetDateTime,
    ) -> Result<(), SubscriptionsError> {
        let id = Uuid::now_v7();
        sqlx::query!(
            r#"
            INSERT INTO subscriptions (
                id, user_id, plan_id, stripe_subscription_id,
                status, cancel_at_period_end,
                current_period_start, current_period_end
            ) VALUES (
                $1, $2, $3, $4,
                $5, $6,
                $7, $8
            )
            ON CONFLICT (stripe_subscription_id) DO UPDATE SET
                status = EXCLUDED.status,
                cancel_at_period_end = EXCLUDED.cancel_at_period_end,
                current_period_start = EXCLUDED.current_period_start,
                current_period_end = EXCLUDED.current_period_end,
                updated_at = now()
            "#,
            id,
            user_id.as_uuid(),
            plan_id.as_uuid(),
            stripe_subscription_id,
            status,
            cancel_at_period_end,
            current_period_start,
            current_period_end,
        )
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    /// Refresh status + period + cancel flag for a known Stripe sub.
    /// Used by `customer.subscription.updated` and `invoice.paid`.
    pub async fn update_state_in_tx<'c>(
        &self,
        tx: &mut sqlx::Transaction<'c, sqlx::Postgres>,
        stripe_subscription_id: &str,
        status: &str,
        cancel_at_period_end: bool,
        current_period_start: OffsetDateTime,
        current_period_end: OffsetDateTime,
    ) -> Result<u64, SubscriptionsError> {
        let res = sqlx::query!(
            r#"
            UPDATE subscriptions
            SET status = $2,
                cancel_at_period_end = $3,
                current_period_start = $4,
                current_period_end = $5,
                updated_at = now()
            WHERE stripe_subscription_id = $1
            "#,
            stripe_subscription_id,
            status,
            cancel_at_period_end,
            current_period_start,
            current_period_end,
        )
        .execute(&mut **tx)
        .await?;
        Ok(res.rows_affected())
    }

    /// `customer.subscription.deleted` handler. Marks the row canceled and
    /// stamps `canceled_at`. Returns the user_id so the caller can revoke
    /// subscription-source enrollments in the same tx.
    pub async fn mark_canceled_in_tx<'c>(
        &self,
        tx: &mut sqlx::Transaction<'c, sqlx::Postgres>,
        stripe_subscription_id: &str,
    ) -> Result<Option<UserId>, SubscriptionsError> {
        let row = sqlx::query!(
            r#"
            UPDATE subscriptions
            SET status = 'canceled',
                canceled_at = now(),
                updated_at = now()
            WHERE stripe_subscription_id = $1
            RETURNING user_id
            "#,
            stripe_subscription_id,
        )
        .fetch_optional(&mut **tx)
        .await?;
        Ok(row.map(|r| UserId::from_uuid(r.user_id)))
    }

    pub async fn count_active(&self) -> Result<i64, SubscriptionsError> {
        let row = sqlx::query!(
            r#"
            SELECT COUNT(*) AS "count!"
            FROM subscriptions
            WHERE status IN ('trialing','active','past_due')
            "#,
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(row.count)
    }

    pub async fn find_active_for_user(
        &self,
        user_id: UserId,
    ) -> Result<Option<Subscription>, SubscriptionsError> {
        let row = sqlx::query!(
            r#"
            SELECT id, user_id, plan_id, stripe_subscription_id, status,
                   cancel_at_period_end, current_period_start,
                   current_period_end, canceled_at
            FROM subscriptions
            WHERE user_id = $1
              AND status IN ('trialing','active','past_due','paused','scheduled_cancel')
            ORDER BY current_period_end DESC
            LIMIT 1
            "#,
            user_id.as_uuid()
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| Subscription {
            id: SubscriptionId::from_uuid(r.id),
            user_id: UserId::from_uuid(r.user_id),
            plan_id: PlanId::from_uuid(r.plan_id),
            stripe_subscription_id: r.stripe_subscription_id,
            status: r.status,
            cancel_at_period_end: r.cancel_at_period_end,
            current_period_start: r.current_period_start,
            current_period_end: r.current_period_end,
            canceled_at: r.canceled_at,
        }))
    }
}
