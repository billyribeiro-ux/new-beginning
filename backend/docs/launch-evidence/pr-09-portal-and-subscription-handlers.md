# PR #9 — Customer Portal + invoice.paid + subscription.* + renewal flow

**Date:** 2026-05-25
**Scope:** BACKEND.md §21 row 9.

## Gates

| Gate | Result |
|---|---|
| fmt / clippy `-D warnings` / sqlx-prepare-check | ✅ |
| `cargo test --workspace` | ✅ 86/86 |
| `cargo deny check` | ✅ |
| `POST /v1/billing/portal` returns `{url}` (Recording fake) | wired |
| `invoice.paid` extends `current_period_end` + inserts invoice + audit row | ✅ |
| `customer.subscription.updated` flips `cancel_at_period_end` | ✅ |
| `customer.subscription.deleted` revokes ONLY subscription-source enrollments | ✅ |

## invoice.paid → renewal

```
before:  current_period_end = 2026-06-24 23:51:19+00
POST invoice.paid (period_end = NOW + 60d)
after:   current_period_end = 2026-07-25 00:04:31+00      ← +30 days
         invoices: in_test_pr8, in_test_renew
         audit_log: action='subscription.renewed' inv='in_test_renew'
```

## customer.subscription.updated → cancel-at-period-end flip

```
POST { cancel_at_period_end: true }
subscriptions.cancel_at_period_end:  false → true     ✅
status remains 'active'                              ✅ (cancel happens at period end)
```

## customer.subscription.deleted → revoke subscription-source enrollments

```
before:
  enrollments: 1 purchase active + 1 subscription active

POST customer.subscription.deleted

after:
  subscriptions: status='canceled', canceled_at IS NOT NULL  ✅
  enrollments:   purchase=active    ← survived (BACKEND.md §8.5)
                 subscription=inactive ← revoked
  audit_log: action='subscription.canceled',
             metadata.subscription_source_enrollments_revoked=1  ✅
```

## Anti-regressions (added in PR #9)

45. `customer.subscription.deleted` revokes ONLY `source='subscription'`
    enrollments. `purchase`-source enrollments are forever (refund is the
    only path to revoke them — PR #15).
46. `invoice.paid` is the renewal signal — it extends
    `current_period_end` AND inserts an `invoices` row in ONE tx.
    Stripe is the timing source of truth; our cron never schedules a
    "renew at period end" job (BACKEND.md §1.7).
47. `customer.subscription.updated` is the single source of truth for
    `cancel_at_period_end` / `status` / period bounds. Portal-driven
    plan switches / pauses arrive here.
48. `POST /v1/billing/portal` reuses the user's existing
    `stripe_customer_id` when present; otherwise creates one on-the-fly
    via the same idempotency-key scheme.
