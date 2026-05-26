# PR #14 — Admin endpoints (KPIs + leads + messages + products + customers + manual grant)

## Scope (BACKEND.md §11, §21 PR #14)

Eight new admin routes, all gated by `require_admin`:

1. `GET /v1/admin/stats` — revenue_30d_cents, orders_30d,
   active_subscriptions, signups_30d, leads_30d.
2. `GET /v1/admin/leads?limit=200` — newest first.
3. `GET /v1/admin/messages?status=&limit=200` — closed status set.
4. `PATCH /v1/admin/messages/{id}` — status change; audited.
5. `GET /v1/admin/products?kind=` — includes inactive.
6. `PATCH /v1/admin/products/{id}` — `active` toggle (audited as
   `admin.product.activated` / `admin.product.deactivated`).
7. `GET /v1/admin/customers?q=&limit=50` — email substring search via
   `ILIKE %q%` over `users.email::citext`.
8. `POST /v1/admin/customers/{id}/grant-entitlement` — manual license
   issuance + (for course-kind) enrollment + audit row. One tx.

Plus storage repo additions:
- `LeadsRepo::list`, `count_since`.
- `ContactRepo::list`, `update_status`.
- `OrdersRepo::list_recent`, `revenue_since`.
- `UsersRepo::count_since`, `search_by_email`.
- `SubscriptionsRepo::count_active`.
- `ProductsRepo::list_all`, `set_active`.

## Why this shape

- **`require_admin` at the router layer**, not per-handler — single
  enforcement point; a new admin endpoint inherits the gate by being in
  the `admin` Router.
- **Audit + mutate in one tx, every time** — BACKEND.md §22 rule 8.
  Both messages PATCH and products PATCH demonstrate the pattern (open
  tx → UPDATE → if 0 rows rollback + 404 → audit::record_in_tx → commit).
- **Manual grants synthesize a fresh `OrderId`** as `source_ref_id` —
  the licenses repo's "skip existing" check keys off
  `(user, product, source_ref_id)`. Real orders carry a real
  `OrderId`; manual grants carry a synthetic one so admin re-issuances
  don't collide with paid-order issuance.
- **Manual grant requires a `reason: String`** (4–500 chars) — written
  to the audit row. Every "why did this account suddenly get an
  indicator?" question is answerable from `audit_log`.
- **`UsersRepo::search_by_email` uses `ILIKE %q%`** — small data set
  (TradeFlex is in the thousands of users for the foreseeable future);
  the citext column makes the comparison case-insensitive without
  function-side overhead. When the user table hits ~100k a trigram
  index lands.
- **`StatsResponse.active_subscriptions` is a count, not MRR cents** —
  computing MRR requires `subscription_plans.price_cents` populated,
  which it isn't until PR #14 admin plan CRUD lands. Documented in
  evidence + notes; the dashboard surfaces "active subs" + "30-day
  revenue" today.

## Anti-regressions 63–66

63. The admin `Router::route_layer(require_admin)` MUST stay outside the
    `merge(authed)` tree. Mixing them would let a `require_auth`
    middleware run twice (one of them as the wrong gate), or worse,
    miss admin gating on an endpoint that gets dropped into the wrong
    sub-tree.
64. `customers::grant_entitlement` MUST synthesize a fresh `OrderId`
    per grant. Reusing a constant would make the second grant a no-op
    (the skip-existing check would match), making the audit row claim
    a license that wasn't actually issued.
65. `messages::patch` MUST rollback the tx if `UPDATE` returns 0 rows
    affected. Otherwise the audit row claims a status change that
    never happened.
66. `products::patch` MUST audit BOTH directions (`activated` /
    `deactivated`). A single `admin.product.toggled` action would hide
    the new state from the audit log and require a JSON lookup to
    decode — keeping the action name asymmetric makes grep/alert
    rules trivially writeable.

## Runtime evidence

`crates/http/tests/admin_grant_integration.rs` walks the manual-grant
flow end-to-end at the storage layer:
- Creates Alice + an admin actor user + a course product.
- Runs the exact `issue_for_purchase_in_tx` + `create_for_purchase_in_tx`
  + `audit::record_in_tx` sequence the handler uses.
- Asserts the license is active, the enrollment row exists, the audit
  row carries actor_user_id + reason + product_slug.
- Re-grants with a fresh synthetic order id — asserts a NEW license
  with a distinct id is minted (anti-regression #64).

```
$ DATABASE_URL=… cargo test -p tradeflex-http --test admin_grant_integration
running 1 test
test admin_grant_writes_license_enrollment_and_audit ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Gates

```
$ cargo fmt --all                                            # clean
$ SQLX_OFFLINE=true cargo clippy --all-targets -- -D warnings   # clean
$ DATABASE_URL=… cargo test --workspace                      # 90+ tests pass
$ cargo deny check                                           # clean
```

## What's NOT in PR #14

- **Full product CRUD** (create / replace / delete / duplicate). Only
  the `active` toggle ships today. Editing `specs_json`, `price_cents`,
  `highlights`, etc. lands once the admin UI ships an editor pane —
  there's no API consumer for it before then. The storage layer's
  `find_by_id` / `list_all` are ready.
- **Plans CRUD** (`/v1/admin/plans` + `set_active` on
  `subscription_plans`). Same reasoning; will need `stripe_price_id`
  validation against Stripe at create time.
- **Order detail + refund trigger.** The refund flow ships in PR #15.
  `OrdersRepo::list_recent` is the read-side primitive — the handler
  for `GET /v1/admin/orders` is trivially derivable; deferred to the
  refund PR where the admin order detail view becomes load-bearing.
- **CSV stream for leads** (`/v1/admin/leads.csv`). The
  `list(limit)` primitive supports it; the streaming response wrapper
  + the `text/csv` Content-Type handler is a follow-up.
- **Customer detail** (`/v1/admin/customers/{id}` with orders + subs +
  contact history). Same reasoning as orders detail.
- **Recent leads / messages / orders shortcuts** on the dashboard
  KPI route. The dashboard can fetch the three list endpoints with
  small limits; bundling them into one response adds latency for no
  payload benefit.

## Note on stats

`active_subscriptions` is a count, not MRR cents. The MRR computation
requires `subscription_plans.price_cents` populated for real (today
those are placeholder values from the seeder). When PR #14's plan CRUD
extension lands, the KPI handler will add `mrr_cents` alongside the
count — keeping the count is still useful as a leading indicator.
