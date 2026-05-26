//! `OrdersRepo` + `OrderItemsRepo` (in one file — they're always mutated
//! together).
//!
//! BACKEND.md §4 0008–0009. PR #7 ships only `create_pending`; the
//! `paid`/`refunded` transitions land in PR #8 / #15.
//!
//! The pending-order row is created in the api-binary's `/v1/checkout`
//! handler BEFORE the Stripe call: that way a Stripe error doesn't strand
//! the user without a row to retry against, and `client_reference_id` on
//! the Stripe Checkout Session can carry our `order_id` straight to the
//! webhook handler.

use common::ids::{OrderId, PlanId, ProductId, UserId};
use sqlx::types::Json;
use sqlx::PgPool;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

/// One cart line as serialized into `orders.cart_snapshot`. Persisted as
/// JSONB so the BFF can replay the cart shape when reconstructing receipts.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CartSnapshotLine {
    pub kind: String, // "product" | "plan"
    pub slug: String,
    pub name: String,
    pub price_cents: i64,
    pub quantity: i64,
}

/// Items as inserted into `order_items`. `product_id` xor `plan_id` is
/// enforced by the table CHECK constraint.
#[derive(Debug, Clone)]
pub struct NewOrderItem {
    pub product_id: Option<ProductId>,
    pub plan_id: Option<PlanId>,
    pub quantity: i32,
    pub unit_price_cents: i64,
    pub line_total_cents: i64,
    pub name_snapshot: String,
    pub slug_snapshot: String,
}

#[derive(Debug, Clone)]
pub struct Order {
    pub id: OrderId,
    pub user_id: UserId,
    pub status: String, // 'pending'/'paid'/'refunded'/'failed' — CHECK constraint
    pub subtotal_cents: i64,
    pub tax_cents: i64,
    pub total_cents: i64,
    pub currency: String,
    pub stripe_checkout_session_id: Option<String>,
    pub created_at: OffsetDateTime,
}

#[derive(Clone)]
pub struct OrdersRepo {
    pool: PgPool,
}

#[derive(Debug, thiserror::Error)]
pub enum OrdersError {
    #[error("order line items mismatch (subtotal vs line sum)")]
    SubtotalMismatch,
    #[error("sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
}

impl OrdersRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Insert a `pending` order with its line items, all inside ONE tx so a
    /// partial failure can never strand a header row without items.
    ///
    /// Pending orders auto-expire after `pending_ttl` — PR #11's worker
    /// runs a cron that flips long-pending rows to `failed`.
    #[allow(clippy::too_many_arguments)]
    pub async fn create_pending(
        &self,
        user_id: UserId,
        currency: &str,
        subtotal_cents: i64,
        tax_cents: i64,
        items: &[NewOrderItem],
        cart_snapshot: &[CartSnapshotLine],
        pending_ttl: Duration,
    ) -> Result<Order, OrdersError> {
        let total = subtotal_cents
            .checked_add(tax_cents)
            .ok_or(OrdersError::SubtotalMismatch)?;
        let line_sum: i64 = items.iter().map(|i| i.line_total_cents).sum();
        if line_sum != subtotal_cents {
            return Err(OrdersError::SubtotalMismatch);
        }

        let mut tx = self.pool.begin().await?;
        let id = Uuid::now_v7();
        let expires_at = OffsetDateTime::now_utc() + pending_ttl;
        let snapshot_json = serde_json::to_value(cart_snapshot)
            .map_err(|e| OrdersError::Sqlx(sqlx::Error::Decode(Box::new(e))))?;
        let row = sqlx::query!(
            r#"
            INSERT INTO orders (
                id, user_id, status,
                subtotal_cents, tax_cents, total_cents, currency,
                cart_snapshot, expires_at
            ) VALUES (
                $1, $2, 'pending',
                $3, $4, $5, $6,
                $7, $8
            )
            RETURNING id, user_id, status,
                      subtotal_cents, tax_cents, total_cents, currency,
                      stripe_checkout_session_id, created_at
            "#,
            id,
            user_id.as_uuid(),
            subtotal_cents,
            tax_cents,
            total,
            currency,
            snapshot_json,
            expires_at,
        )
        .fetch_one(&mut *tx)
        .await?;

        for item in items {
            let item_id = Uuid::now_v7();
            sqlx::query!(
                r#"
                INSERT INTO order_items (
                    id, order_id, product_id, plan_id,
                    quantity, unit_price_cents, line_total_cents,
                    name_snapshot, slug_snapshot
                ) VALUES (
                    $1, $2, $3, $4,
                    $5, $6, $7,
                    $8, $9
                )
                "#,
                item_id,
                row.id,
                item.product_id.map(|p| p.as_uuid()),
                item.plan_id.map(|p| p.as_uuid()),
                item.quantity,
                item.unit_price_cents,
                item.line_total_cents,
                item.name_snapshot,
                item.slug_snapshot,
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(Order {
            id: OrderId::from_uuid(row.id),
            user_id: UserId::from_uuid(row.user_id),
            status: row.status,
            subtotal_cents: row.subtotal_cents,
            tax_cents: row.tax_cents,
            total_cents: row.total_cents,
            currency: row.currency,
            stripe_checkout_session_id: row.stripe_checkout_session_id,
            created_at: row.created_at,
        })
    }

    /// After the Stripe Checkout Session is minted, attach its id so the
    /// webhook handler can resolve the order from the session.
    pub async fn attach_stripe_checkout_session(
        &self,
        order_id: OrderId,
        session_id: &str,
    ) -> Result<(), OrdersError> {
        sqlx::query!(
            r#"
            UPDATE orders
            SET stripe_checkout_session_id = $2, updated_at = now()
            WHERE id = $1
            "#,
            order_id.as_uuid(),
            session_id,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Look up by the Stripe Checkout Session id. Hot path in the
    /// `checkout.session.completed` handler.
    pub async fn find_by_stripe_checkout_session(
        &self,
        session_id: &str,
    ) -> Result<Option<Order>, OrdersError> {
        let row = sqlx::query!(
            r#"
            SELECT id, user_id, status,
                   subtotal_cents, tax_cents, total_cents, currency,
                   stripe_checkout_session_id, created_at
            FROM orders
            WHERE stripe_checkout_session_id = $1
            "#,
            session_id,
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| Order {
            id: OrderId::from_uuid(r.id),
            user_id: UserId::from_uuid(r.user_id),
            status: r.status,
            subtotal_cents: r.subtotal_cents,
            tax_cents: r.tax_cents,
            total_cents: r.total_cents,
            currency: r.currency,
            stripe_checkout_session_id: r.stripe_checkout_session_id,
            created_at: r.created_at,
        }))
    }

    /// Flip a pending order to `paid` and stamp `paid_at = now()`. Used
    /// inside the same tx that creates entitlements + invoice +
    /// audit_log — atomic by construction.
    pub async fn mark_paid_in_tx<'c>(
        &self,
        tx: &mut sqlx::Transaction<'c, sqlx::Postgres>,
        order_id: OrderId,
        stripe_payment_intent_id: Option<&str>,
    ) -> Result<(), OrdersError> {
        sqlx::query!(
            r#"
            UPDATE orders
            SET status = 'paid',
                paid_at = now(),
                stripe_payment_intent_id = COALESCE($2, stripe_payment_intent_id),
                updated_at = now()
            WHERE id = $1 AND status = 'pending'
            "#,
            order_id.as_uuid(),
            stripe_payment_intent_id,
        )
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    /// Flip a paid order to `'refunded'`. Idempotent: a second call is a
    /// no-op because the WHERE clause requires status = 'paid'. Returns
    /// `(user_id, total_cents, currency)` for downstream handler use
    /// (audit row + receipt email). `None` if the order was not found
    /// or wasn't paid (already refunded → idempotent re-drive).
    pub async fn mark_refunded_in_tx<'c>(
        &self,
        tx: &mut sqlx::Transaction<'c, sqlx::Postgres>,
        stripe_payment_intent_id: &str,
    ) -> Result<Option<(OrderId, UserId, i64, String)>, OrdersError> {
        let row = sqlx::query!(
            r#"
            UPDATE orders
            SET status = 'refunded',
                refunded_at = now(),
                updated_at = now()
            WHERE stripe_payment_intent_id = $1
              AND status = 'paid'
            RETURNING id, user_id, total_cents, currency
            "#,
            stripe_payment_intent_id,
        )
        .fetch_optional(&mut **tx)
        .await?;
        Ok(row.map(|r| {
            (
                OrderId::from_uuid(r.id),
                UserId::from_uuid(r.user_id),
                r.total_cents,
                r.currency,
            )
        }))
    }

    /// Read every order_items row for the given order. Used by the
    /// dispatcher to issue entitlements.
    pub async fn list_items(&self, order_id: OrderId) -> Result<Vec<OrderItemRow>, OrdersError> {
        let rows = sqlx::query!(
            r#"
            SELECT id, order_id, product_id, plan_id,
                   quantity, unit_price_cents, line_total_cents,
                   name_snapshot, slug_snapshot
            FROM order_items
            WHERE order_id = $1
            "#,
            order_id.as_uuid()
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| OrderItemRow {
                product_id: r.product_id.map(ProductId::from_uuid),
                plan_id: r.plan_id.map(PlanId::from_uuid),
                quantity: r.quantity,
                slug_snapshot: r.slug_snapshot,
                name_snapshot: r.name_snapshot,
            })
            .collect())
    }

    /// Admin dashboard "recent orders" feed. Newest first; filtered
    /// optionally by status. Cap at the call site.
    pub async fn list_recent(
        &self,
        status_filter: Option<&str>,
        limit: i64,
    ) -> Result<Vec<Order>, OrdersError> {
        let rows = sqlx::query!(
            r#"
            SELECT id, user_id, status,
                   subtotal_cents, tax_cents, total_cents, currency,
                   stripe_checkout_session_id, created_at
            FROM orders
            WHERE $1::text IS NULL OR status = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
            status_filter,
            limit,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| Order {
                id: OrderId::from_uuid(r.id),
                user_id: UserId::from_uuid(r.user_id),
                status: r.status,
                subtotal_cents: r.subtotal_cents,
                tax_cents: r.tax_cents,
                total_cents: r.total_cents,
                currency: r.currency,
                stripe_checkout_session_id: r.stripe_checkout_session_id,
                created_at: r.created_at,
            })
            .collect())
    }

    /// Aggregated revenue for paid orders since a cutoff. Returned as a
    /// `(count, sum_cents)` pair so the admin KPI handler can compute MRR
    /// proxies + total revenue in one round-trip.
    pub async fn revenue_since(&self, since: OffsetDateTime) -> Result<(i64, i64), OrdersError> {
        let row = sqlx::query!(
            r#"
            SELECT COUNT(*) AS "count!",
                   COALESCE(SUM(total_cents), 0)::bigint AS "sum_cents!"
            FROM orders
            WHERE status = 'paid' AND created_at >= $1
            "#,
            since,
        )
        .fetch_one(&self.pool)
        .await?;
        Ok((row.count, row.sum_cents))
    }

    pub async fn find_by_id(&self, id: OrderId) -> Result<Option<Order>, OrdersError> {
        let row = sqlx::query!(
            r#"
            SELECT id, user_id, status,
                   subtotal_cents, tax_cents, total_cents, currency,
                   stripe_checkout_session_id, created_at
            FROM orders
            WHERE id = $1
            "#,
            id.as_uuid()
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| Order {
            id: OrderId::from_uuid(r.id),
            user_id: UserId::from_uuid(r.user_id),
            status: r.status,
            subtotal_cents: r.subtotal_cents,
            tax_cents: r.tax_cents,
            total_cents: r.total_cents,
            currency: r.currency,
            stripe_checkout_session_id: r.stripe_checkout_session_id,
            created_at: r.created_at,
        }))
    }
}

/// One row out of `order_items`, projected for the dispatcher.
#[derive(Debug, Clone)]
pub struct OrderItemRow {
    pub product_id: Option<ProductId>,
    pub plan_id: Option<PlanId>,
    pub quantity: i32,
    pub slug_snapshot: String,
    pub name_snapshot: String,
}

// `Json<T>` is `serde_json::Value` for the cart snapshot column; here we
// just re-export the alias so callers don't need to depend on sqlx::types.
pub type CartSnapshotJson = Json<Vec<CartSnapshotLine>>;
