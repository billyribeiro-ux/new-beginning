# PR #7 — stripe-client + /v1/checkout + webhook receiver (skeleton)

**Date:** 2026-05-25
**Scope:** BACKEND.md §21 row 7.

## Gates

| Gate | Result |
|---|---|
| `cargo fmt --check` | ✅ |
| `SQLX_OFFLINE=true cargo clippy --workspace --all-targets -- -D warnings` | ✅ |
| `cargo sqlx prepare --workspace --check` | ✅ |
| `cargo test --workspace` | ✅ **83/83** (+9 new: idempotency, webhook sig, recording fake) |
| `cargo deny check` | ✅ |
| Webhook signature: missing / wrong-key / stale / valid → 400/400/400/200 | ✅ |
| Webhook replay deduped → 200 with no extra dispatch | ✅ |
| Crash-recovery: re-POST of `processed_at IS NULL` row flips it to processed | ✅ |
| Reconciliation query (`processed_at IS NULL AND received_at < now() - …`) returns the stranded row | ✅ |

## Webhook signature verification

```
$ curl -X POST .../v1/webhooks/stripe   (no sig header)
HTTP 400

$ curl -X POST .../v1/webhooks/stripe -H 'stripe-signature: t=NOW,v1=BAD' -d '{...}'
HTTP 400

$ curl -X POST .../v1/webhooks/stripe -H 'stripe-signature: t=10min-ago,v1=...' -d '{...}'
HTTP 400   # stale

$ curl -X POST .../v1/webhooks/stripe -H 'stripe-signature: t=NOW,v1=GOOD' -d '{...}'
HTTP 200   # processed

$ curl … same event again …
HTTP 200   # deduped
```

All four 4xx cases log a `WARN` line with the specific `SignatureError`
variant; the 200 cases log an `INFO` line including event_id + kind.

## Dedupe + crash recovery (BACKEND.md §8.3 load-bearing)

The `stripe_events.claim` query uses Postgres' `xmax = 0` trick to
distinguish a fresh INSERT from a recognized-existing UPDATE:

```sql
INSERT INTO stripe_events (event_id, event_type, payload, received_at)
VALUES ($1, $2, $3, now())
ON CONFLICT (event_id) DO UPDATE SET event_id = stripe_events.event_id
RETURNING (xmax = 0) AS "freshly_inserted!", processed_at
```

```
=== Simulate a crashed dispatch (insert with processed_at = NULL) ===
INSERT 0 1

=== Reconciliation query finds it ===
   event_id    | event_type   | attempts | age      | unprocessed
 evt_pr7_crash | invoice.paid |     0    | 00:05:00 | t

=== Re-POST the same event (sig-verified) — handler re-drives ===
HTTP 200

=== Row is now processed ===
   event_id    | event_type   | attempts | processed
 evt_pr7_crash | invoice.paid |     1    | t
 evt_pr7_one   | checkout…    |     1    | t
```

The `reconcile_stripe_events` cron (PR #11) calls
`StripeEventsRepo::list_unprocessed_older_than(120, …)` to surface these
rows and re-runs the handler. PR #7 demonstrates the row state +
re-drivability; the cron is a one-line addition once the worker binary
exists.

## stripe-client surface (PR #7 slice)

| Trait method | PR # ship | Status |
|---|---|---|
| `get_or_create_customer_for_user` | PR #7 | Real `reqwest` impl ✅ |
| `create_checkout_session` | PR #7 | Real `reqwest` impl ✅ |
| `cancel_subscription_at_period_end` | PR #9 | — |
| `refund_payment_intent` | PR #15 | — |
| `create_customer_portal_session` | PR #9 | — |
| webhook signature verify | PR #7 | Real impl ✅ |

The `RecordingStripeApi` test fake captures every call into an in-memory
log — used by integration tests that don't have real Stripe test keys.

## Anti-regressions (added in PR #7)

34. Stripe writes carry an `Idempotency-Key` header derived from
    `(env, user_id, action, nonce)`. Retries collapse to a single Stripe
    write.
35. Webhook signature uses HMAC-SHA256 over `"{t}.{raw_body}"`,
    constant-time-compared. Stale-timestamp tolerance is 5 min.
36. The webhook receiver always returns 200 unless signature is invalid
    or claim fails (BACKEND.md §8.3). A failed dispatch does NOT bounce
    Stripe; the reconciliation cron re-drives.
37. `stripe_events.claim` is the ONE primitive for idempotency
    decisions. Handlers don't make their own dedupe judgments.
38. `dispatch()` is a noop in PR #7 and ALWAYS runs in the same tx as
    `mark_processed_in_tx`. A crash anywhere in this window leaves
    `processed_at = NULL`, which is the safe state for the
    reconciliation cron.
39. The wire-level Stripe client is built with `reqwest`, not
    `async-stripe`. See BACKEND_NOTES PR #7 for the rationale.
