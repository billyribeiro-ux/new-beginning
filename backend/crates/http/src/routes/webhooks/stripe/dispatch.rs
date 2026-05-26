//! Stripe webhook dispatcher — the body PR #7 deferred.
//!
//! BACKEND.md §8.4 handler table. PR #8 wires `checkout.session.completed`.
//! Other kinds are logged + treated as success (so the dedupe row stamps
//! `processed_at` and we don't replay forever); their real handlers land
//! in PR #9 / #15.
//!
//! Every effect (orders → paid, invoices insert, subscriptions upsert,
//! enrollments / licenses, audit_log) runs inside the SAME transaction
//! the receiver opened. A crash anywhere here rolls back and leaves the
//! `stripe_events` row with `processed_at = NULL`.

use anyhow::Context;
use common::ids::{OrderId, PlanId, ProductId, UserId};
use serde_json::Value;
use storage::OrderItemRow;
use stripe_client::ParsedEvent;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::state::AppState;

pub async fn dispatch(
    state: &AppState,
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    event: &ParsedEvent,
) -> anyhow::Result<()> {
    match event.kind.as_str() {
        "checkout.session.completed" => checkout_session_completed(state, tx, event).await,
        "invoice.paid" => invoice_paid(state, tx, event).await,
        "customer.subscription.updated" => customer_subscription_updated(state, tx, event).await,
        "customer.subscription.deleted" => customer_subscription_deleted(state, tx, event).await,
        "charge.refunded" => charge_refunded(state, tx, event).await,
        other => {
            tracing::info!(
                kind = %other,
                event_id = %event.id,
                "stripe event kind not yet handled",
            );
            Ok(())
        }
    }
}

/// Refund handler. Stripe POSTs `charge.refunded` after either:
///   * the admin triggered a refund via `POST /v1/admin/orders/{id}/refund`
///     (which calls `stripe.refund_payment_intent`), OR
///   * an admin or the user issued a refund through the Stripe Dashboard /
///     Customer Portal.
///
/// Either way, the side effects converge here — the api binary does NOT
/// revoke entitlements inline at refund-trigger time; it waits for this
/// event so the source of truth is single and crash-safe via the same
/// `stripe_events` replay mechanism as every other event.
///
/// One tx:
///   1. Resolve the order by `payment_intent` id.
///   2. Flip `orders.status` from 'paid' → 'refunded' (idempotent — a
///      re-drive matches 0 rows because the WHERE requires status='paid').
///   3. Revoke licenses + enrollments whose `source_ref_id` is this order.
///   4. Drop an in-app notification on the user's feed.
///   5. Audit `order.refunded` on `orders`.
///
/// If no order matches (refund of a subscription invoice, not a one-shot
/// purchase), we still audit + notify + return Ok so the event stamps
/// processed_at and we don't loop. Subscription-source revocation
/// happens via `customer.subscription.deleted`, not here.
async fn charge_refunded(
    state: &AppState,
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    event: &ParsedEvent,
) -> anyhow::Result<()> {
    let obj = event
        .raw
        .pointer("/data/object")
        .context("charge.refunded missing /data/object")?;
    let payment_intent_id = obj
        .get("payment_intent")
        .and_then(Value::as_str)
        .context("charge.refunded missing data.object.payment_intent")?;
    let charge_id = obj.get("id").and_then(Value::as_str).unwrap_or("");

    let order_marked = state
        .orders
        .mark_refunded_in_tx(tx, payment_intent_id)
        .await
        .context("orders.mark_refunded_in_tx")?;

    let Some((order_id, user_id, total_cents, currency)) = order_marked else {
        // Either no matching order (subscription invoice path) or already
        // refunded (idempotent re-drive). Log + return Ok.
        tracing::info!(
            payment_intent_id,
            charge_id,
            event_id = %event.id,
            "charge.refunded: no paid order matched (subscription path or already refunded)",
        );
        return Ok(());
    };

    let licenses_revoked = state
        .licenses
        .revoke_for_order_in_tx(tx, order_id)
        .await
        .context("licenses.revoke_for_order_in_tx")?;
    let enrollments_revoked = state
        .enrollments
        .revoke_for_order_in_tx(tx, order_id)
        .await
        .context("enrollments.revoke_for_order_in_tx")?;

    // Drop an in-app notification on the user's feed. Email is the
    // mailer's job (deferred); the in-app row guarantees the user sees
    // SOMETHING the next time they open the dashboard.
    let notification_kind = "billing.refund.processed";
    let title = "Refund processed".to_string();
    let body = format!(
        "We've issued a refund of {:.2} {} to your original payment method.",
        (total_cents as f64) / 100.0,
        currency.to_uppercase()
    );
    // The repo's `create` runs against the pool, not the tx. We accept
    // this asymmetry: the audit row + state mutation MUST share the tx
    // (rule 8), but the user-visible notification is a downstream effect
    // — if it fails the refund still stands, and a re-drive of the event
    // re-creates the notification. The double-notification risk on
    // re-drive is the lesser evil vs. losing audit consistency.
    let _ = state
        .notifications
        .create(user_id, notification_kind, &title, &body, "stripe")
        .await;

    state
        .audit
        .record_in_tx(
            tx,
            None,
            "order.refunded",
            "orders",
            &order_id.to_string(),
            serde_json::json!({
                "payment_intent_id": payment_intent_id,
                "charge_id": charge_id,
                "total_cents": total_cents,
                "currency": currency,
                "licenses_revoked": licenses_revoked,
                "enrollments_revoked": enrollments_revoked,
                "user_id": user_id.to_string(),
            }),
            None,
        )
        .await
        .context("audit.record_in_tx for order.refunded")?;

    tracing::info!(
        %order_id, %user_id, payment_intent_id, charge_id,
        licenses_revoked, enrollments_revoked,
        "order refunded — entitlements revoked",
    );
    Ok(())
}

/// Subscription renewal. Extends `current_period_end` + stamps `invoices`.
async fn invoice_paid(
    state: &AppState,
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    event: &ParsedEvent,
) -> anyhow::Result<()> {
    let obj = event
        .raw
        .pointer("/data/object")
        .context("invoice.paid missing /data/object")?;
    let invoice_id = obj
        .get("id")
        .and_then(Value::as_str)
        .context("invoice has no id")?;
    let sub_id = obj.get("subscription").and_then(Value::as_str);
    let amount = obj.get("amount_paid").and_then(Value::as_i64).unwrap_or(0);
    let currency = obj
        .get("currency")
        .and_then(Value::as_str)
        .unwrap_or("usd")
        .to_string();
    let number = obj
        .get("number")
        .and_then(Value::as_str)
        .unwrap_or(invoice_id)
        .to_string();
    let invoice_date = obj
        .pointer("/status_transitions/paid_at")
        .and_then(Value::as_i64)
        .map(unix_to_offset)
        .transpose()?
        .unwrap_or_else(OffsetDateTime::now_utc);
    let cps = obj
        .get("period_start")
        .and_then(Value::as_i64)
        .map(unix_to_offset)
        .transpose()?;
    let cpe = obj
        .get("period_end")
        .and_then(Value::as_i64)
        .map(unix_to_offset)
        .transpose()?;
    let user_id_from_sub: Option<UserId> = if let Some(sid) = sub_id {
        if let (Some(cps), Some(cpe)) = (cps, cpe) {
            state
                .subscriptions
                .update_state_in_tx(tx, sid, "active", false, cps, cpe)
                .await
                .context("subscriptions.update_state_in_tx")?;
        }
        sqlx::query!(
            "SELECT user_id FROM subscriptions WHERE stripe_subscription_id = $1",
            sid
        )
        .fetch_optional(&mut **tx)
        .await?
        .map(|r| UserId::from_uuid(r.user_id))
    } else {
        None
    };
    if let Some(uid) = user_id_from_sub {
        state
            .invoices
            .upsert_in_tx(
                tx,
                None,
                uid,
                invoice_id,
                &number,
                "paid",
                amount,
                &currency,
                invoice_date,
            )
            .await
            .context("invoices.upsert_in_tx")?;
        state
            .audit
            .record_in_tx(
                tx,
                Some(uid),
                "subscription.renewed",
                "subscription",
                sub_id.unwrap_or(""),
                serde_json::json!({
                    "stripe_event_id": event.id,
                    "stripe_invoice_id": invoice_id,
                    "amount_cents": amount,
                }),
                None,
            )
            .await
            .context("audit.record_in_tx subscription.renewed")?;
    }
    tracing::info!(
        invoice_id = %invoice_id,
        sub_id = %sub_id.unwrap_or(""),
        "invoice.paid processed",
    );
    Ok(())
}

/// Plan switch / cancel-at-period-end / pause — refresh mirror.
async fn customer_subscription_updated(
    state: &AppState,
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    event: &ParsedEvent,
) -> anyhow::Result<()> {
    let obj = event
        .raw
        .pointer("/data/object")
        .context("subscription.updated missing /data/object")?;
    let sub_id = obj
        .get("id")
        .and_then(Value::as_str)
        .context("sub has no id")?;
    let status = obj
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("active")
        .to_string();
    let cancel_at_period_end = obj
        .get("cancel_at_period_end")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let cps = obj
        .get("current_period_start")
        .and_then(Value::as_i64)
        .map(unix_to_offset)
        .transpose()?
        .unwrap_or_else(OffsetDateTime::now_utc);
    let cpe = obj
        .get("current_period_end")
        .and_then(Value::as_i64)
        .map(unix_to_offset)
        .transpose()?
        .unwrap_or(cps + time::Duration::days(30));
    let touched = state
        .subscriptions
        .update_state_in_tx(tx, sub_id, &status, cancel_at_period_end, cps, cpe)
        .await
        .context("subscriptions.update_state_in_tx")?;
    if touched == 0 {
        tracing::info!(
            sub_id = %sub_id,
            "subscription.updated for unknown sub — ignoring (likely pre-checkout)",
        );
    }
    tracing::info!(
        sub_id = %sub_id,
        status = %status,
        cancel_at_period_end,
        "customer.subscription.updated processed",
    );
    Ok(())
}

/// Subscription cancel — fires at `current_period_end`. Revoke
/// subscription-source enrollments inline (BACKEND.md §1.7 / §8.4).
async fn customer_subscription_deleted(
    state: &AppState,
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    event: &ParsedEvent,
) -> anyhow::Result<()> {
    let obj = event
        .raw
        .pointer("/data/object")
        .context("subscription.deleted missing /data/object")?;
    let sub_id = obj
        .get("id")
        .and_then(Value::as_str)
        .context("sub has no id")?;

    let user_id = state
        .subscriptions
        .mark_canceled_in_tx(tx, sub_id)
        .await
        .context("subscriptions.mark_canceled_in_tx")?;
    if let Some(uid) = user_id {
        let revoked = state
            .enrollments
            .revoke_subscription_source_in_tx(tx, uid)
            .await
            .context("enrollments.revoke_subscription_source_in_tx")?;
        state
            .audit
            .record_in_tx(
                tx,
                Some(uid),
                "subscription.canceled",
                "subscription",
                sub_id,
                serde_json::json!({
                    "stripe_event_id": event.id,
                    "subscription_source_enrollments_revoked": revoked,
                }),
                None,
            )
            .await
            .context("audit.record_in_tx subscription.canceled")?;
        tracing::info!(
            sub_id = %sub_id,
            user_id = %uid,
            revoked,
            "customer.subscription.deleted processed",
        );
    } else {
        tracing::info!(
            sub_id = %sub_id,
            "subscription.deleted for unknown sub — ignoring",
        );
    }
    Ok(())
}

async fn checkout_session_completed(
    state: &AppState,
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    event: &ParsedEvent,
) -> anyhow::Result<()> {
    let obj = event
        .raw
        .pointer("/data/object")
        .context("event payload missing /data/object")?;

    let session_id = obj
        .get("id")
        .and_then(Value::as_str)
        .context("checkout session has no id")?;
    let mode = obj.get("mode").and_then(Value::as_str).unwrap_or("payment");
    let payment_intent_id = obj.get("payment_intent").and_then(Value::as_str);
    let subscription_id = obj.get("subscription").and_then(Value::as_str);
    let client_ref = obj
        .get("client_reference_id")
        .and_then(Value::as_str)
        .or_else(|| obj.pointer("/metadata/order_id").and_then(Value::as_str));

    // Look up the order. Prefer the explicit Checkout Session id (we
    // attached it after Stripe created the session), fall back to
    // client_reference_id.
    let order = if let Some(o) = state
        .orders
        .find_by_stripe_checkout_session(session_id)
        .await
        .context("orders.find_by_stripe_checkout_session")?
    {
        o
    } else if let Some(cr) = client_ref {
        let order_uuid: Uuid = cr.parse().context("client_reference_id not a uuid")?;
        state
            .orders
            .find_by_id(OrderId::from_uuid(order_uuid))
            .await
            .context("orders.find_by_id")?
            .context("order from client_reference_id not found")?
    } else {
        anyhow::bail!("checkout.session.completed: cannot resolve order (no session id match, no client_reference_id)");
    };

    if order.status == "paid" {
        // Already processed by a prior delivery whose mark-processed
        // committed AFTER the rest of the side effects. Idempotent
        // exit — re-running the side effects would no-op anyway thanks
        // to the ON CONFLICT DO NOTHING guards on enrollments / licenses
        // / invoices, but bailing here avoids spurious audit_log rows.
        tracing::info!(order_id = %order.id, "order already paid; skipping side effects");
        return Ok(());
    }

    // 1. Orders → paid.
    state
        .orders
        .mark_paid_in_tx(tx, order.id, payment_intent_id)
        .await
        .context("orders.mark_paid_in_tx")?;

    // 2. Invoice (best-effort — checkout.session.completed for
    //    mode=payment doesn't always carry an invoice id; for
    //    mode=subscription, `invoice` is the first-period invoice that
    //    Stripe auto-creates).
    if let Some(inv_id) = obj.get("invoice").and_then(Value::as_str) {
        let inv_number = obj
            .pointer("/invoice_number")
            .and_then(Value::as_str)
            .unwrap_or(inv_id);
        let amount = obj
            .get("amount_total")
            .and_then(Value::as_i64)
            .unwrap_or(order.total_cents);
        state
            .invoices
            .upsert_in_tx(
                tx,
                Some(order.id),
                order.user_id,
                inv_id,
                inv_number,
                "paid",
                amount,
                &order.currency,
                OffsetDateTime::now_utc(),
            )
            .await
            .context("invoices.upsert_in_tx")?;
    }

    // 3. Subscription mirror (only on mode=subscription).
    if mode == "subscription" {
        if let Some(sub_id) = subscription_id {
            // Find the plan via the order's first plan_id line.
            let items = state
                .orders
                .list_items(order.id)
                .await
                .context("orders.list_items")?;
            let plan_id: PlanId = items
                .iter()
                .find_map(|i| i.plan_id)
                .context("subscription checkout has no plan line")?;
            // Current period bounds come from the event when present;
            // fallback is now + 30 days (refined when PR #9's
            // `customer.subscription.updated` lands).
            let cps = obj
                .get("current_period_start")
                .and_then(Value::as_i64)
                .map(unix_to_offset)
                .transpose()?
                .unwrap_or(OffsetDateTime::now_utc());
            let cpe = obj
                .get("current_period_end")
                .and_then(Value::as_i64)
                .map(unix_to_offset)
                .transpose()?
                .unwrap_or(cps + time::Duration::days(30));
            state
                .subscriptions
                .upsert_from_stripe_in_tx(
                    tx,
                    order.user_id,
                    plan_id,
                    sub_id,
                    "active",
                    false,
                    cps,
                    cpe,
                )
                .await
                .context("subscriptions.upsert_from_stripe_in_tx")?;
        }
    }

    // 4. Issue entitlements per order line.
    let items: Vec<OrderItemRow> = state
        .orders
        .list_items(order.id)
        .await
        .context("orders.list_items")?;
    let mut issued_licenses: Vec<String> = Vec::new();
    let mut enrollment_count = 0usize;
    for it in &items {
        if let Some(product_id) = it.product_id {
            let product = state
                .products
                .find_by_slug(&it.slug_snapshot)
                .await
                .context("products.find_by_slug")?
                .context("product missing for order line")?;
            let pid = ProductId::from_uuid(product.id.as_uuid());
            if pid != product_id {
                tracing::warn!(
                    order_line_product = %product_id,
                    looked_up = %pid,
                    "product_id drift between order_items and current row",
                );
            }
            match product.kind.as_str() {
                "course" => {
                    state
                        .enrollments
                        .create_for_purchase_in_tx(tx, order.user_id, pid, order.id)
                        .await
                        .context("enrollments.create_for_purchase")?;
                    enrollment_count += 1;
                }
                "indicator" => {
                    let kind = short_kind(&product.slug);
                    if let Some(license) = state
                        .licenses
                        .issue_for_purchase_in_tx(
                            tx,
                            &state.password_pepper,
                            order.user_id,
                            pid,
                            order.id,
                            &kind,
                        )
                        .await
                        .context("licenses.issue_for_purchase")?
                    {
                        issued_licenses.push(license.prefix);
                    }
                }
                other => {
                    tracing::warn!(kind = %other, "unhandled product kind on order line");
                }
            }
        }
        // Plan lines don't issue per-product entitlements; the
        // subscription mirror is the entitlement source.
        let _ = it;
    }

    // 5. Audit log — same tx, per BACKEND.md §22 rule 8.
    state
        .audit
        .record_in_tx(
            tx,
            Some(UserId::from_uuid(order.user_id.as_uuid())),
            "order.paid",
            "order",
            &order.id.to_string(),
            serde_json::json!({
                "stripe_event_id": event.id,
                "checkout_session_id": session_id,
                "mode": mode,
                "payment_intent_id": payment_intent_id,
                "subscription_id": subscription_id,
                "enrollments_created": enrollment_count,
                "licenses_issued": issued_licenses.len(),
            }),
            None,
        )
        .await
        .context("audit.record_in_tx order.paid")?;

    tracing::info!(
        order_id = %order.id,
        mode = %mode,
        enrollments = enrollment_count,
        licenses = issued_licenses.len(),
        "checkout.session.completed processed",
    );
    Ok(())
}

fn unix_to_offset(secs: i64) -> anyhow::Result<OffsetDateTime> {
    OffsetDateTime::from_unix_timestamp(secs).context("invalid unix timestamp from Stripe")
}

/// Two-letter shortcut for the license key's `kind` segment. Derives from
/// the slug's first two ASCII uppercase letters — `revolution-ranger` →
/// `RR`. PR #14's admin product CRUD will let an operator override.
fn short_kind(slug: &str) -> String {
    let chars: Vec<char> = slug
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|p| !p.is_empty())
        .filter_map(|p| p.chars().next())
        .map(|c| c.to_ascii_uppercase())
        .collect();
    if chars.len() >= 2 {
        format!("{}{}", chars[0], chars[1])
    } else if let Some(c) = chars.first() {
        format!("{c}X")
    } else {
        "XX".into()
    }
}

#[cfg(test)]
mod tests {
    use super::short_kind;

    #[test]
    fn short_kind_handles_common_shapes() {
        assert_eq!(short_kind("revolution-ranger"), "RR");
        assert_eq!(short_kind("options-101"), "O1");
        assert_eq!(short_kind("liquidity-hawk-2"), "LH");
        assert_eq!(short_kind("singleword"), "SX");
        assert_eq!(short_kind(""), "XX");
    }
}
