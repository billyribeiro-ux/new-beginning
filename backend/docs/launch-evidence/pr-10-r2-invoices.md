# PR #10 — R2 client + invoice PDF endpoints

## Scope (BACKEND.md §1.4, §8.7, §9, §21 PR #10)

1. **`r2-client` crate** — new workspace member exporting:
   - `ObjectStore` trait: `presigned_get`, `put_object`, `head_object`, `delete`.
   - `R2Client` real implementation built on `rusty-s3` (SigV4 presign) + `reqwest`. R2's S3-compatible endpoint with path-style URLs, `auto` region.
   - `RecordingObjectStore` in-memory fake: deterministic `https://r2.test/fake/{key}?ttl={secs}` URLs; HashMap-backed.
   - `keys` module: stable key conventions (`invoices/{user_id}/{invoice_id}.pdf`, etc.).
2. **`InvoicesRepo` extension** — two new methods:
   - `attach_pdf_r2_key(&str, &str)` — UPDATE … WHERE stripe_invoice_id = $1 AND (pdf_r2_key IS NULL OR pdf_r2_key = $2). Idempotent on retry, **never overwrites a different key** (would orphan the prior object).
   - `find_for_user(UserId, InvoiceId)` — SELECT … WHERE id = $1 AND user_id = $2. The user_id predicate **is** the entitlement check.
3. **Two new authed routes**:
   - `GET /v1/billing/invoices` — list user's invoices, exposes `has_pdf` boolean (not the raw key).
   - `GET /v1/billing/invoices/{id}/pdf-url` — entitlement-checked HEAD + 300s presigned GET.
4. **`AppState.r2`** new `Arc<dyn ObjectStore>` field.
5. **api binary** — constructs `R2Client` from `R2_*` env vars; in dev (non-production) falls back to `RecordingObjectStore` with a WARN log. In production, missing R2_* env vars **fail-closed at boot** (BACKEND.md §9).

## Why this shape

- **`rusty-s3` over `aws-sdk-s3`** — ~50 dependencies lighter, single-purpose (SigV4 presigning + minimal action types). The full AWS SDK is unnecessary for a workload that only PUTs PDFs and presigns GETs.
- **HEAD before sign** in `pdf_url` — catches the "DB row says we have it, but R2 lost it" case so we 404 cleanly instead of handing the user a URL that 403s mid-download.
- **`has_pdf: bool` in list response, not the raw `pdf_r2_key`** — the key is internal infrastructure; the UI only needs to know whether to show the download link.
- **PDF presign TTL = 300s** — short enough that leaked URLs have a narrow blast radius; long enough for a normal click-to-download. The TTL is mirrored to the BFF in the response so it can set cache headers.
- **Production fail-closed on missing R2_***  — the recording fallback exists for local dev only. A silent production fallback would mean uploads vanish into RAM, lost on the next restart.

## Idempotency guarantees (verified by integration test)

| Operation | Repeat behavior |
|---|---|
| `InvoicesRepo::upsert_in_tx` with the same stripe_invoice_id | ON CONFLICT DO NOTHING — no duplicate row. |
| `InvoicesRepo::attach_pdf_r2_key(invoice, key)` called twice with same key | OK — still matches, no error, pdf_r2_key unchanged. |
| `InvoicesRepo::attach_pdf_r2_key(invoice, different_key)` after key already set | **0 rows affected**, existing key preserved (no orphan). |
| `find_for_user(wrong_user_id, invoice_id)` | `None` — entitlement enforced. |

## Runtime evidence

`crates/http/tests/billing_invoices_integration.rs` is the PR #10 evidence test. It:

1. Inserts users Alice + Bob via `UsersRepo::create`.
2. Inserts an invoice for Alice via `InvoicesRepo::upsert_in_tx` (twice — proves idempotency).
3. PUTs a fake PDF into a `RecordingObjectStore` and `attach_pdf_r2_key`s it.
4. Re-attaches with the SAME key (idempotent — no error).
5. Attempts to overwrite with a **different** key — asserts 0 rows affected, original key preserved.
6. Looks up the invoice as Bob — asserts `None` (entitlement check works).
7. Presigns a GET, asserts the URL embeds the key and TTL.
8. HEAD on the present key succeeds; HEAD on a missing key returns `StoreError::NotFound` (the variant the handler maps to 404).

```
$ DATABASE_URL=postgres://tradeflex:tradeflex@127.0.0.1:5435/tradeflex \
  cargo test -p tradeflex-http --test billing_invoices_integration

running 1 test
test full_pr10_flow_storage_plus_recording_r2 ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Gates

```
$ cargo fmt --all                                          # clean
$ SQLX_OFFLINE=true cargo clippy --all-targets -- -D warnings   # clean
$ DATABASE_URL=… cargo test --workspace                    # 90+ tests pass, 0 failed
$ cargo deny check                                          # advisories ok, bans ok, licenses ok, sources ok
```

## What's NOT in PR #10

- The **worker** that actually fetches the Stripe invoice PDF and uploads it to R2. That lives in PR #11 (worker binary). Until then, `pdf_r2_key` stays NULL for every webhook-inserted invoice, so the UI shows the row in the list but hides the download link — exactly the behavior `has_pdf: false` is designed to surface.
- **Account-deletion** R2 object cleanup — PR #16.
- **Data export** writing to R2 — PR #16.

## Anti-regression notes

- `attach_pdf_r2_key`'s WHERE clause is load-bearing: dropping the `pdf_r2_key IS NULL OR pdf_r2_key = $2` predicate would let a retry with a wrong key silently orphan the original PDF in R2. The "0 rows affected" branch in the integration test is the explicit regression guard.
- `find_for_user` is the only path handlers should use to resolve an invoice by id. A bare `find_by_id` would skip the entitlement check; **no such method exists**, by design.
- The HEAD before presign in `pdf_url` exists because R2 → DB consistency is best-effort: if the worker uploaded then crashed before DB commit, the row says NULL (handler returns 404 from `find_for_user` branch); if R2 dropped the object after a successful upload (impossible in practice but defensible), HEAD catches it instead of leaving the user a broken URL.
