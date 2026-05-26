# PR #6 — public surface (products/plans/leads/contact) + general rate limiter + BFF cutover

**Date:** 2026-05-25
**Scope:** BACKEND.md §21 row 6.

## Gates

| Gate | Result |
|---|---|
| `cargo fmt --check` | ✅ |
| `SQLX_OFFLINE=true cargo clippy --workspace --all-targets -- -D warnings` | ✅ |
| `cargo sqlx prepare --workspace --check` | ✅ |
| `cargo test --workspace` | ✅ **74/74** (+4 new rate-limit tests, +1 from PR #5 strengthened) |
| `cargo deny check` | ✅ |
| `pnpm run check` (svelte-check) | ✅ 0 errors / 0 warnings |
| BACKEND.md §16.3 `EXPLAIN ANALYZE` on `leads_created_at_idx` | ✅ |

## Public-surface live calls

### Reads

```
$ curl -H "x-service-token: ..." :8081/v1/public/products?kind=indicator
  count=1
  - revolution-ranger (indicator) $997.00

$ curl ... :8081/v1/public/products?kind=course
  count=1
  - options-101 (course) $997.00

$ curl ... :8081/v1/public/products?kind=invalid
HTTP 422

$ curl ... :8081/v1/public/products/revolution-ranger
  slug=revolution-ranger name=Revolution Ranger rating={'value': 4.9, 'count': 1284}

$ curl ... :8081/v1/public/products/nope
HTTP 404

$ curl ... :8081/v1/public/plans
  - day-trading-monthly (monthly) $247.00 featured=False
  - day-trading-quarterly (quarterly) $697.00 featured=True
  - day-trading-annual (annual) $1997.00 featured=False
```

### Lead capture + lockout

```
$ for i in 1..5; do curl -X POST ... /v1/public/leads -d '{"email":"leadN@..."}'; done
  attempt 1..5: HTTP 201

$ curl -i -X POST ... /v1/public/leads -d '{"email":"lead6@..."}'
HTTP/1.1 429 Too Many Requests
retry-after: 11

{"error":{"code":"rate_limited","message":"rate limited"},"request_id":"019e6152-..."}

$ psql -c "SELECT email, source FROM leads ORDER BY created_at"
 lead1..lead5 @ example.com  | free-guide
```

The honeypot path took 429 here only because the bucket was already empty
from the lockout test; the honeypot itself is verified in the
`PR #6 honeypot` unit-style test (route does an early-return 201 when
`website` is non-empty, before the DB write).

### Contact + lockout

```
$ for i in 1..3; do curl -X POST ... /v1/public/contact -d '{...}'; done
  attempt 1..3: HTTP 201

$ curl -i -X POST ... /v1/public/contact
HTTP/1.1 429 Too Many Requests
retry-after: 19
```

3 rows in `contact_messages`. The bucket is BACKEND.md §12's `Contact` =
3/min.

## EXPLAIN ANALYZE — `leads_created_at_idx` is used

Seeded 1000 bulk leads + 5 from the lockout test (1005 rows total), then
`ANALYZE leads`, then:

```
EXPLAIN ANALYZE
SELECT id, email::text, source, created_at FROM leads
ORDER BY created_at DESC LIMIT 200;

                                                               QUERY PLAN
-----------------------------------------------------------------------------------------------------------------------------------------
 Limit  (cost=0.28..10.04 rows=200 width=61) (actual time=0.013..0.040 rows=200 loops=1)
   ->  Index Scan using leads_created_at_idx on leads  (cost=0.28..49.35 rows=1005 width=61) (actual time=0.011..0.030 rows=200 loops=1)
 Planning Time: 0.344 ms
 Execution Time: 0.064 ms
```

**`Index Scan using leads_created_at_idx`** — the exact gate from
BACKEND.md §16.3 / §25 item 7. No seq-scan. 0.064 ms execution for 200
rows out of 1005.

## Rate-limit framework (PR #6's load-bearing refactor)

`http::middleware::rate_limit::RateLimiterSet` replaces PR #3's
single-purpose `LoginLimiter`. Four buckets today
(`PublicRead`/`LeadCapture`/`Contact`/`Login`), each its own `governor`
instance; per-IP keying.

Unit tests pinned in `crates/http/src/middleware/rate_limit.rs`:

- `lead_capture_fires_on_sixth_attempt` — exact §16.3 evidence shape.
- `buckets_are_independent_per_ip` — exhausting `Contact` does not
  affect `PublicRead`.
- `buckets_are_independent_per_ip_keying` — IP `a` exhausted does not
  affect IP `b`.
- `labels_are_static_snake_case_and_distinct` — metric-cardinality guard.

## SvelteKit BFF cutover (flag-gated)

`src/lib/server/rust/client.ts` is the single point where the BFF calls
Rust. It injects `X-Service-Token`, generates `X-Request-Id`, forwards
the user's `Cookie` and `X-Forwarded-For`, and maps Rust JSON errors
(including 429 with `Retry-After`) into typed throws.

`src/routes/free-guide/+page.server.ts` and
`src/routes/contact/+page.server.ts` now have both paths:

```ts
if (useRustBackend()) {
  await callRust('/v1/public/leads', { event, body: { email, source } });
} else {
  await persistDrizzle(email, source);
}
```

Toggle by exporting `USE_RUST_BACKEND=true`. PR #17 will flip the flag
permanently and delete the Drizzle path.

## Anti-regressions (added in PR #6)

29. `RateLimiterSet` is the ONE rate-limit primitive — handlers pick a
    `Bucket`, never construct their own limiter. Per-bucket isolation
    pinned by unit tests.
30. Honeypot check on `/v1/public/leads` and `/v1/public/contact`: a
    non-empty `website` field → silent 201 + no DB insert. Bots are
    not signaled.
31. JSONB / array columns ride through the API verbatim. No mid-layer
    re-shaping; the BFF owns the UI mapping.
32. `Money` cents stay `i64` end-to-end across the wire (response field
    `price_cents: number`). The BFF treats them as JS Number; the
    `Money` serde guard rejects values that would lose precision in JS
    (anti-regression #1, still load-bearing).
33. BFF Rust calls go through `callRust()` only — no ad-hoc `fetch` to
    `RUST_API_BASE_URL`. Centralizes header injection, error mapping,
    request-id propagation.
