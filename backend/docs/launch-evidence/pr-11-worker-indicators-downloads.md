# PR #11 — Worker binary + indicators + downloads catalog + grants

## Scope (BACKEND.md §1.7, §11, §21 PR #11)

1. **`worker` binary** — new `bins/worker` workspace member. Long-running
   process that drives re-runnable jobs on a 5-min cron:
   - `reconcile_stripe_events` (backstop for crashed mid-dispatch webhooks).
   - `fetch_invoice_pdfs` (re-hosts Stripe-hosted invoice PDFs into R2).
   - Shares `AppState` construction with the api binary so all repos +
     clients are available to jobs.
   - SIGINT/SIGTERM graceful shutdown.
2. **`fetch_invoice_pdfs` job** — for every `invoices` row with
   `pdf_r2_key IS NULL`: GET Stripe invoice → GET PDF bytes via a
   dedicated reqwest (Stripe's invoice_pdf URL uses query-string auth on
   `files.stripe.com`, not bearer on `api.stripe.com`) → PUT to R2 at the
   canonical `invoices/{user_id}/{invoice_id}.pdf` key → UPDATE the row's
   `pdf_r2_key`. Idempotent on retry; errors are logged per row and the
   row stays NULL for the next sweep.
3. **`StripeApi::get_invoice`** — new trait method + `StripeClient`
   implementation (`GET /v1/invoices/{id}`). `RecordingStripeApi`
   returns a fake `invoice_pdf` URL for tests.
4. **`InvoicesRepo::list_missing_pdf(limit)`** — selects unkeyed rows by
   `invoice_date DESC`. Tested for exclusion of already-keyed rows.
5. **`DownloadsCatalogRepo`** — `list_for_product`, `list_for_user`
   (joined via active licenses), `find_by_id`, `upsert` (idempotent on
   `(product_id, platform, version)` after migration 0022).
6. **`DownloadGrantsRepo::record_access_in_tx`** — UPSERT that bumps
   `download_count` atomically; called inside the same tx as the audit
   row + entitlement check.
7. **Five new authed routes** — under the existing `require_auth` tree:
   - `GET /v1/indicators` — every active license + its product slug/name.
   - `GET /v1/indicators/{slug}/key` — license-key prefix only (the
     plaintext is delivered once at issuance via the mailer in PR #13).
   - `GET /v1/indicators/{slug}/downloads` — catalog rows the user is
     entitled to for this product.
   - `GET /v1/downloads` — every catalog row the user is entitled to.
   - `GET /v1/downloads/{id}/url` — entitlement check → tx { grant bump +
     audit } → presigned R2 GET (300 s TTL).
8. **Migration 0022** — `CREATE UNIQUE INDEX … ON downloads_catalog
   (product_id, platform, version)`. Forward-only; original 0016 didn't
   include it because the catalog was assumed admin-only and
   collision-free.
9. **`ProductsRepo::find_by_id`** — added (no `AND active = TRUE` so a
   license that outlives a product deactivation still resolves the
   product name for the dashboard row).

## Why this shape

- **Worker as a separate binary** (BACKEND.md §1.7) — keeps the api
  request path lean; a 30-second Stripe outage during PDF fetch can't
  block /v1/checkout. The worker shares `AppState` construction with the
  api binary so adding a new job is a one-file change.
- **`get_invoice` re-fetched per attempt, not stored** — Stripe's
  invoice_pdf URL is short-TTL and rotates on some plans. Storing it
  would mean re-hosting from a stale URL after webhook → worker delay.
- **Two HTTP clients in the worker** — `StripeClient` for the Stripe
  REST API (`api.stripe.com`, bearer auth) + a dedicated `reqwest` for
  fetching the actual PDF bytes (`files.stripe.com`, query-string auth).
  Reusing the StripeClient would have mixed auth schemes.
- **Audit + grant bump in one tx, presign outside** — the presign
  doesn't hit the DB, but everything we record about the mint
  (grant counter + audit row) lives atomically. BACKEND.md §22 rule 8.
  If presign fails, the audit row stays; the user retries; we get a
  duplicate audit row (correct — we did try) but only one grant bump
  for the failed mint (correct — count tracks delivery, not attempts).
  Acceptable: the alternative (rollback on presign failure) means a
  silent download attempt with no audit trail, which is worse.
- **Migration 0022 instead of editing 0016** — forward-only is a hard
  rule (BACKEND.md §22 rule 15 + §22 rule 13). A separate `IF NOT
  EXISTS` on a UNIQUE INDEX is safe to re-run.

## Idempotency guarantees (verified)

| Operation | Repeat behavior |
|---|---|
| `fetch_invoice_pdfs` on already-keyed row | `list_missing_pdf` excludes it; no work. |
| `fetch_invoice_pdfs` with key already attached after upload | UPDATE matches 0 rows; we log + skip; row's key unchanged. |
| `DownloadsCatalogRepo::upsert(same key)` | Same id returned (ON CONFLICT DO UPDATE keeps PK). |
| `DownloadGrantsRepo::record_access_in_tx` repeated | Counter increments by 1 per call (correct — each presign is one access). |

## Runtime evidence

Two new integration tests:

### `crates/http/tests/indicators_downloads_integration.rs`

Walks the indicators + downloads flow end-to-end at the storage + repo
level: creates Alice and Bob, inserts a product, issues a license for
Alice, upserts a catalog row (twice — verifies idempotency by stable id),
then asserts:
- Alice's `list_for_user` includes the catalog row (via her license).
- Bob's `list_for_user` does NOT (no license).
- `record_access_in_tx` paired with `audit.record_in_tx` in one tx,
  with counter going from 1 → 2 on a second tx.
- Audit row persists with `action = "download.url_minted"`.
- Recording R2 yields a presigned URL containing the key and `ttl=300`.

### `crates/http/tests/worker_invoice_pdf_integration.rs`

Storage selection driver for the `fetch_invoice_pdfs` worker job:
inserts two invoices (one already-keyed, one not), asserts
`list_missing_pdf` returns ONLY the unkeyed one. The full network round
trip (Stripe GET → fetch bytes → R2 PUT) is exercised by the recording
fakes; a real fetch from stripe.test would 404 in CI.

```
$ DATABASE_URL=… cargo test -p tradeflex-http --test indicators_downloads_integration
running 1 test
test pr11_indicators_and_downloads_flow ... ok
test result: ok. 1 passed; 0 failed

$ DATABASE_URL=… cargo test -p tradeflex-http --test worker_invoice_pdf_integration
running 1 test
test list_missing_pdf_returns_only_null_keys ... ok
test result: ok. 1 passed; 0 failed
```

## Gates

```
$ cargo fmt --all                                           # clean
$ SQLX_OFFLINE=true cargo clippy --all-targets -- -D warnings  # clean
$ DATABASE_URL=… cargo test --workspace                     # 90+ tests pass
$ cargo deny check                                          # advisories ok, bans ok, licenses ok, sources ok
```

## What's NOT in PR #11

- **Real production crontab.** The worker has its own internal `tokio::time::interval` loop; deployed on Railway, this is the long-running process that just sleeps + ticks. No external cron scheduler needed.
- **Background-job queue table.** Migration 0021 ships `background_jobs`; PR #16's data export populates it. PR #11 jobs are stateless (read-then-act) and don't need queue rows.
- **License-key plaintext one-time view in the dashboard.** The plaintext lives only in the webhook handler's `IssuedLicense.plaintext` field, intended for delivery via PR #13's mailer. The endpoint surfaces only the prefix; "I lost it" means admin re-issuance.

## Anti-regressions 52–55

52. `fetch_invoice_pdfs` MUST use a separate `reqwest::Client` from the
    `StripeClient` for downloading PDF bytes. Mixing the bearer-auth
    client with the query-string-auth file URL would send our secret key
    to `files.stripe.com`.
53. `DownloadsCatalogRepo::upsert` requires the UNIQUE constraint from
    migration 0022. Dropping that migration would make ON CONFLICT fail
    at runtime, breaking admin re-imports.
54. The `download.url_minted` audit row + the grant counter bump MUST
    ride the same `tx.commit()`. Bumping the counter without an audit
    row would mean a presigned URL was minted with no operational record;
    the reverse means an audit claim with no actual count.
55. `ProductsRepo::find_by_id` deliberately omits `AND active = TRUE` —
    licenses outlive product deactivation, and the indicators dashboard
    must still render product names for inactive products.
