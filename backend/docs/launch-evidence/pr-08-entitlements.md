# PR #8 — checkout.session.completed handler + entitlements

**Date:** 2026-05-25
**Scope:** BACKEND.md §21 row 8.

## Gates

| Gate | Result |
|---|---|
| fmt / clippy `-D warnings` / sqlx-prepare-check | ✅ |
| `cargo test --workspace` | ✅ **86/86** (+3 new: license-key shape, short_kind) |
| `cargo deny check` | ✅ |
| Webhook dispatcher: indicator purchase → license issued | ✅ `TF-RR-…` issued, audit row written |
| Webhook dispatcher: course purchase → enrollment created | ✅ |
| Webhook dispatcher: subscription checkout → subscriptions mirror + invoice | ✅ active row + `in_test_pr8` invoice |
| Replay of processed event → 200, NO new license / enrollment | ✅ count unchanged |
| GET /v1/subscription returns the active mirror row | ✅ |

## Indicator purchase

```
INSERT pending orders + order_items via SQL (simulates /v1/checkout)
POST /v1/webhooks/stripe   { id:"evt_pr8_checkout_one", type:"checkout.session.completed",
                             data.object: { id:"cs_test_pr8", mode:"payment",
                                            payment_intent:"pi_test_pr8",
                                            client_reference_id:"<order>",
                                            metadata.order_id:"<order>" } }
HTTP 200

orders.status = paid                  ✅
licenses.count(active = TRUE) = 1     ✅
licenses.prefix LIKE 'TF-RR-…'        ✅
audit_log.action = 'order.paid'       ✅
audit_log.metadata.licenses_issued = 1  ✅
```

## Course purchase

```
POST same shape with the Options 101 product line
HTTP 200

enrollments.count(active = TRUE) = 1
licenses.count = 1   (unchanged — only the indicator created a license)
audit_log.metadata.enrollments_created = 1
```

## Subscription purchase

```
POST checkout.session.completed with
   data.object: { mode:"subscription",
                  subscription:"sub_test_pr8",
                  invoice:"in_test_pr8",
                  current_period_start:NOW, current_period_end:NOW+30d,
                  client_reference_id:<order with plan_id line> }

subscriptions: 1 active row (sub_test_pr8, status=active, cancel_at_period_end=false)
invoices: in_test_pr8 / amount_cents=24700 / status=paid
audit_log.metadata.mode = "subscription"
```

## Replay dedupe

```
POST evt_pr8_checkout_one again (already-processed)
HTTP 200

licenses.count: 1 → 1   (NO new license; second call hits the
                         "duplicate stripe event already processed" branch)
```

log line:
```
{"level":"INFO","fields":{"message":"duplicate stripe event already processed, skipping",
  "event_id":"evt_pr8_checkout_one","kind":"checkout.session.completed"}}
```

## GET /v1/subscription

```
$ curl -b cookies -H 'x-service-token: …' :8081/v1/subscription
{
  "subscription": {
    "id": "019e618c-…",
    "plan_id": "019e603a-…",
    "stripe_subscription_id": "sub_test_pr8",
    "status": "active",
    "cancel_at_period_end": false,
    "current_period_start": "2026-05-25T23:51:19Z",
    "current_period_end":   "2026-06-24T23:51:19Z"
  }
}
```

## Anti-regressions (added in PR #8)

40. `dispatch.rs` is the ONE place that handles event-type → side-effects.
    Webhook receiver + `reconcile_stripe_events` cron call the SAME fn.
41. `order.paid` audit_log row is written in the same tx as the
    enrollments / licenses inserts. A rollback removes all four
    consistently.
42. Idempotency comes from THREE layers in concert:
    (a) `stripe_events.claim` (event-id dedupe — webhook receiver),
    (b) order.status != 'pending' early-return (handler-level),
    (c) `ON CONFLICT (user_id, product_id) DO NOTHING` on enrollments
        + the "skip if existing license for source_ref_id" guard.
    Any one is enough; three is defense in depth.
43. License keys are `TF-{kind}-{4x4-Crockford}`. Hash via Argon2id +
    same keyed pepper as passwords. Plaintext returned to handler ONCE;
    DB stores hash + 8-char prefix only.
44. `subscriptions` is the ONE source of truth for sub state on our
    side. Stripe is the upstream truth; the mirror gets re-synced by
    PR #9's `customer.subscription.updated` handler.
