# PR #15 — Refund flow

## Scope (BACKEND.md §15, §21 PR #15)

1. **`POST /v1/admin/orders/{id}/refund`** — admin trigger. Resolves
   payment_intent, calls `StripeApi::refund_payment_intent` with an
   Idempotency-Key derived from `(env, actor, "order_refund", order_id)`,
   audits `admin.refund.triggered`. **Does NOT revoke inline.**
2. **`charge.refunded` webhook handler** — single source of truth. In
   one tx: `mark_refunded_in_tx` (flips `'paid'`→`'refunded'`, returns
   `(order, user, total, currency)`) → revoke licenses + enrollments by
   `source_ref_id = order_id` → audit `order.refunded`. Drops a
   notification on the user's feed (outside the tx — re-drive may dupe;
   the lesser evil vs. losing audit consistency).
3. **`StripeApi::refund_payment_intent`** + `StripeRefund` struct +
   `RecordingStripeApi` mock.
4. **Storage primitives** — `OrdersRepo::mark_refunded_in_tx`,
   `LicensesRepo::revoke_for_order_in_tx`, `EnrollmentsRepo::revoke_for_order_in_tx`.
   All three idempotent (WHERE clauses gate to `status='paid'` /
   `active=TRUE`).

## Why this shape

- **api binary does NOT revoke inline at refund-trigger time.** Source
  of truth is the `charge.refunded` webhook. Even if the admin clicks
  refund and the Stripe call succeeds, we wait for Stripe's POST. This
  preserves crash-safety via the same `stripe_events` replay path as
  every other event. The reconciliation cron (PR #11) re-fetches events
  if Stripe's POST is dropped.
- **Notification outside the tx.** If the notification insert fails we
  log + continue; the refund + entitlement revocation still stand. A
  re-drive of the webhook will create a duplicate notification — that's
  the lesser evil vs. losing audit consistency.
- **Subscription refunds NOT handled here.** Subscription cancellation
  flows through `customer.subscription.deleted` (PR #9). A
  `charge.refunded` against an invoice for a subscription matches no
  order (subscription invoices don't carry `order_id`), so the handler
  logs + returns Ok (event stamps processed, no loop).

## Anti-regressions 67–69

67. `mark_refunded_in_tx`'s WHERE MUST keep `status = 'paid'`. Without
    it, a re-drive would re-revoke and double-audit a finished refund.
68. License + enrollment revoke MUST key off `source_ref_id = order_id`
    AND `source = 'purchase'`. A subscription-source enrollment for the
    same product MUST NOT be revoked by a one-shot refund (sub keeps it
    until `customer.subscription.deleted`).
69. The Idempotency-Key on `refund_payment_intent` MUST include the
    `order_id`. A double-click from the same admin against different
    orders would otherwise collapse to the same Stripe call.

## Runtime evidence

`crates/http/tests/refund_flow_integration.rs`:
- Inserts paid order + license + enrollment + invoice.
- Runs the `mark_refunded_in_tx` + `revoke_for_order_in_tx` (both) +
  audit::record_in_tx sequence.
- Asserts: order in `refunded`, license + enrollment inactive, audit
  row present.
- Re-drives the same sequence → asserts every step is a no-op (0 rows).

```
$ DATABASE_URL=… cargo test -p tradeflex-http --test refund_flow_integration
running 1 test
test refund_revokes_entitlements_and_idempotent_redrive ... ok
test result: ok. 1 passed; 0 failed
```

## Gates

```
$ cargo fmt --all                                            # clean
$ SQLX_OFFLINE=true cargo clippy --all-targets -- -D warnings   # clean
$ DATABASE_URL=… cargo test --workspace                      # 90+ tests pass
$ cargo deny check                                           # clean
```
