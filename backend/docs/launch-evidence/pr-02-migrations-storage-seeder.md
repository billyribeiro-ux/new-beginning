# PR #2 — Migrations + storage::pool + seeder + dump-catalog — Launch Evidence

**Date:** 2026-05-25
**Scope:** BACKEND.md §21 row 2 — Migrations 0001–0021; `storage::pool`;
`seeder` binary; catalog snapshot dump script.

**Verification target:** `just migrate && just seed`; psql verifies row
counts; `/readyz` now reports the DB check.

## Gates

| Gate | Result |
|---|---|
| `cargo fmt --all -- --check` | ✅ |
| `SQLX_OFFLINE=true cargo clippy --workspace --all-targets -- -D warnings` | ✅ |
| `cargo sqlx prepare --workspace --check` | ✅ |
| `cargo test --workspace` | ✅ 36/36 |
| `cargo deny check` | ✅ advisories / bans / licenses / sources all ok |
| `dump-catalog` drift check | ✅ 5 slugs verified |
| `just up && just migrate && just seed` | ✅ |
| psql row counts: 2 products, 3 plans | ✅ |
| `/healthz`, `/readyz` (db up → 200, down → 503, up again → 200) | ✅ |

## Migrations

All 21 SQL files applied in order against fresh Postgres 16:

```
Applied 1/migrate extensions (16.586ms)
Applied 2/migrate users (8.047ms)
Applied 3/migrate sessions (7.353ms)
Applied 4/migrate email verifications (5.214ms)
Applied 5/migrate products (8.325ms)
Applied 6/migrate subscription plans (4.866ms)
Applied 7/migrate subscriptions (5.741ms)
Applied 8/migrate orders (5.530ms)
Applied 9/migrate order items (3.895ms)
Applied 10/migrate invoices (3.513ms)
Applied 11/migrate payment methods (4.246ms)
Applied 12/migrate billing addresses (2.713ms)
Applied 13/migrate enrollments (3.819ms)
Applied 14/migrate course progress (2.730ms)
Applied 15/migrate licenses (4.002ms)
Applied 16/migrate downloads (4.887ms)
Applied 17/migrate notifications (5.436ms)
Applied 18/migrate leads contact (5.175ms)
Applied 19/migrate stripe events (3.592ms)
Applied 20/migrate audit log (3.909ms)
Applied 21/migrate background jobs (4.333ms)
```

`\dt` output confirms 23 tables + `_sqlx_migrations`.

## Seeder run

```
{"timestamp":"…","fields":{"message":"connected, applying migrations before seed"}}
{"timestamp":"…","fields":{"message":"running migrations","migrations":21}}
{"timestamp":"…","fields":{"message":"migrations applied"}}
{"timestamp":"…","fields":{"message":"seeding catalog","indicators":1,"courses":1,"plans":3}}
{"timestamp":"…","fields":{"message":"seed complete"}}
{"timestamp":"…","fields":{"message":"row counts after seed","products":2,"plans":3}}
```

psql verification:

```
       slug        |   kind    |       name        | price_cents | active
-------------------+-----------+-------------------+-------------+--------
 options-101       | course    | Options 101       |       99700 | t
 revolution-ranger | indicator | Revolution Ranger |       99700 | t

         slug          |  cadence  | price_cents | savings_pct | featured
-----------------------+-----------+-------------+-------------+----------
 day-trading-monthly   | monthly   |       24700 |           0 | f
 day-trading-quarterly | quarterly |       69700 |           6 | t
 day-trading-annual    | annual    |      199700 |          33 | f
```

The `featured = true` constraint matches BACKEND.md §1.7: exactly one
featured plan across all cadences. Verified by the
`subscription_plans_one_featured_idx` partial unique index.

## /readyz with DB up/down/up

```
$ curl -i http://127.0.0.1:8081/readyz
HTTP/1.1 200 OK
{"checks":{"db":"ok"},"status":"ready"}

$ docker stop tradeflex-postgres-dev && curl -i http://127.0.0.1:8081/readyz
HTTP/1.1 503 Service Unavailable
{"checks":{"db":"fail: timeout after 100ms"},"status":"degraded"}

$ docker start tradeflex-postgres-dev && curl -i http://127.0.0.1:8081/readyz
HTTP/1.1 200 OK
{"checks":{"db":"ok"},"status":"ready"}
```

BACKEND.md §15 invariant satisfied: 100 ms DB ping; 200 only if reachable;
503 with reason and `AlertKind::ReadinessDegraded` otherwise; auto-recovers.

## Idempotency

Re-running `cargo run -p seeder` does not duplicate rows (every upsert uses
`ON CONFLICT (slug) DO UPDATE`). Confirmed: row counts stayed at 2/3 across
back-to-back invocations.

## Anti-regressions (added in PR #2)

1. `sqlx::query!` macros gate compilation against the schema. `.sqlx/` cache
   is committed; CI fails if it drifts.
2. Migrations are forward-only. No backfills, no edits to shipped files.
3. The `ReadinessRegistry` is additive — PR #8 will register a Stripe check
   without touching `routes/health.rs`.
4. `NUMERIC(3,2)` columns route through `bigdecimal::BigDecimal`, never
   `f32`/`f64` after the boundary.
5. Catalog snapshot in `backend/seeds/catalog.json` is reviewed in PRs;
   `pnpm run dump-catalog` detects drift between it and `src/lib/data/*.ts`.
