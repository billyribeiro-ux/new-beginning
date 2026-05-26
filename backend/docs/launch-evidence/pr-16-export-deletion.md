# PR #16 — Data export + account deletion

## Scope (BACKEND.md §16, §21 PR #16)

1. **`GET /v1/me/export`** — inline JSON dump of every row the user
   owns (user/orders/invoices/licenses/enrollments/subscription/notifications).
   Sufficient for typical GDPR portability; a future background job
   will write gzipped JSON to R2 + email a presigned URL for larger
   footprints.
2. **`DELETE /v1/me`** — soft-delete:
   - `users.deleted_at = now()`, email pseudonymized to
     `deleted-<id>@deleted.tradeflex.invalid`, password_hash + TOTP +
     stripe_customer_id cleared.
   - Every active session for the user purged in the same tx.
   - Audit row `account.deleted`.
   - Idempotent (re-call returns 204 with no change).
3. **`UsersRepo::soft_delete`** — the storage primitive. All side
   effects in one tx; the existing `find_by_email`/`find_by_id` already
   gate on `deleted_at IS NULL`, so the deleted user is invisible
   immediately.

## Why this shape

- **Soft-delete, not hard-delete** — orders / invoices / audit_log are
  retained for legal + accounting reasons (BACKEND.md §16). A retention
  job (later phase) does hard purge after the legally-required window.
- **Pseudonymized email instead of NULL** — the column is NOT NULL +
  UNIQUE; null isn't an option. A pseudonym keeps the constraint happy
  and makes accidental re-use impossible (the `@deleted.tradeflex.invalid`
  suffix would never match a real signup).
- **Inline JSON export, not async-to-R2** — TradeFlex users have small
  footprints (handful of orders + invoices + license keys). When the
  largest user crosses ~5 MB the worker job kicks in. Spec-ready, not
  written today.
- **Session purge in the same tx** — a stolen cookie can't outlive the
  deletion. The user clicks delete → all devices logged out instantly.

## Anti-regressions 70–71

70. `UsersRepo::soft_delete` MUST purge sessions in the same tx as the
    user row update. Otherwise a stolen cookie remains valid until its
    own TTL.
71. `find_by_email` and `find_by_id` MUST keep their `deleted_at IS
    NULL` predicates. Re-introducing a query without that gate would
    let a deleted user log in or appear in admin search.

## Runtime evidence

`crates/http/tests/account_deletion_integration.rs`:
- Creates Alice, asserts visible via find_by_email + find_by_id.
- `soft_delete` returns 1.
- Both find methods return None after deletion.
- Raw row's email ends with `@deleted.tradeflex.invalid`.
- Repeat `soft_delete` returns 0 (idempotent).

```
$ DATABASE_URL=… cargo test -p tradeflex-http --test account_deletion_integration
running 1 test
test soft_delete_pseudonymizes_and_is_idempotent ... ok
test result: ok. 1 passed; 0 failed
```

## Gates

```
$ cargo fmt --all                                            # clean
$ SQLX_OFFLINE=true cargo clippy --all-targets -- -D warnings   # clean
$ DATABASE_URL=… cargo test --workspace                      # 90+ tests pass
$ cargo deny check                                           # clean
```

## What's NOT in PR #16

- **Background-job export to R2.** Not needed at current data scale;
  the inline endpoint covers everything today. The R2 helper + grant
  pattern from PR #11 is the obvious foundation when it lands.
- **Hard-delete retention job.** Deferred — depends on legal team's
  retention window. The soft-deleted state is durable and queryable;
  conversion to hard-delete is a one-time migration once the policy is
  pinned.
- **`account_deletion_scheduled` (delayed-delete with cancel window).**
  The current flow deletes immediately. A future variant with a 7-day
  cooling-off period is documented as a follow-up; today's `DELETE
  /v1/me` is the simpler "the user clicked the button, honor it"
  semantics.
