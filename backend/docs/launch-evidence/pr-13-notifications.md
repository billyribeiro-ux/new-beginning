# PR #13 — Notifications feed + preferences + kind→channel routing

## Scope (BACKEND.md §11, §21 PR #13)

1. **`NotificationsRepo`** — `create`, `list_for_user`, `mark_read`
   (silent no-op on wrong owner — no enumeration leak), `mark_all_read`,
   `unread_count`.
2. **`NotificationPreferencesRepo`** — `get_or_default` (lazy
   materialization via `INSERT … ON CONFLICT DO NOTHING RETURNING`),
   `patch` (partial update with double-Option semantics for DND times so
   the caller can explicitly clear them).
3. **`channels_for_kind`** — pure function mapping a kind slug
   (`billing.invoice.paid`, `course.completed`, …) to
   `(email_opt_in, inapp_opt_in)` from the user's prefs. Unknown / system
   kinds always deliver in-app, never email — the closed set is what
   the UI exposes.
4. **`PreferencesPatch` struct** + `common::deserialize::double_option`
   helper (lifted out of `me/profile.rs` for reuse).
5. **Five new authed routes:**
   - `GET /v1/notifications?unread=true&limit=50`
   - `PATCH /v1/notifications/{id}/read` — silent no-op on wrong owner.
   - `POST /v1/notifications/mark-all-read`
   - `GET /v1/notifications/preferences`
   - `PATCH /v1/notifications/preferences` — validates that
     `dnd_enabled=true` requires `dnd_start` + `dnd_end` present (matches
     the DB CHECK constraint, surfaces as 422 instead of a 500).

## Why this shape

- **Lazy materialization** — we don't write the prefs row at signup.
  Most users never visit the page; the row is created the first time
  `get_or_default` or `patch` runs. `INSERT ... ON CONFLICT DO NOTHING
  RETURNING` is the atomic primitive.
- **`mark_read` returns 204 even on wrong owner** — a 404 would let an
  attacker enumerate valid notification IDs by probing. The `user_id`
  predicate is the entitlement and silently no-ops.
- **`channels_for_kind` is a pure fn** — easy to unit-test exhaustively
  (see `channels_for_kind_routes_correctly`). New kinds are one match
  arm; the unknown-kind fallback is `in-app only` so no event slips
  past with no UI signal.
- **`PreferencesPatch.dnd_start: Option<Option<Time>>`** — true tri-state.
  Default serde would collapse "field absent" and "field=null" both to
  `None`; `double_option` (now in `common::deserialize`, reused from
  `me::profile`) distinguishes them.
- **Handler validates the DND triple before hitting the DB** — the
  schema CHECK `(dnd_enabled=FALSE OR (dnd_start IS NOT NULL AND
  dnd_end IS NOT NULL))` is the final gate, but surfacing it as a 422
  with a clear message is better UX than a DB-error 500.

## Anti-regressions 59–62

59. `mark_read` MUST return 204 (not 404) on wrong-owner. Anti-
    enumeration. The integration test exercises this with `bob` marking
    `alice`'s row.
60. `get_or_default` MUST be one query (the `INSERT … ON CONFLICT DO
    NOTHING RETURNING` + fallback fetch is two; that's still atomic
    because the conflict path means the row exists). A naive
    `SELECT-then-INSERT-if-missing` would race.
61. `channels_for_kind`'s unknown-kind fallback is `(false, true)` —
    in-app only. A `(false, false)` fallback would silently drop
    notifications for system events; a `(true, true)` fallback would
    spam email for them.
62. PATCH-prefs handler MUST pre-validate `dnd_enabled=true → start +
    end present` against the **final** state (current row + patch
    overrides), not just the patch. Otherwise a partial patch could
    flip `dnd_enabled` true without ever sending the times.

## Runtime evidence

`crates/http/tests/notifications_integration.rs` covers:
- Lazy materialization → defaults round-trip (`marketing_email = FALSE`,
  `dnd_enabled = FALSE`, `timezone = "UTC"`).
- Partial patch keeps untouched flags put.
- DND triple round-trips.
- Three rows inserted → mark one read → unread feed excludes it →
  re-mark on already-read returns 0 (idempotent) → bob's `mark_read` on
  alice's row returns 0 (anti-enumeration) → `mark_all_read` clears
  count to 0.

Plus a pure unit test for `channels_for_kind` covering every prefix and
the unknown-kind fallback.

```
$ DATABASE_URL=… cargo test -p tradeflex-http --test notifications_integration
running 2 tests
test channels_for_kind_routes_correctly ... ok
test notifications_full_flow ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Gates

```
$ cargo fmt --all                                            # clean
$ SQLX_OFFLINE=true cargo clippy --all-targets -- -D warnings   # clean
$ DATABASE_URL=… cargo test --workspace                      # 90+ tests pass
$ cargo deny check                                           # clean
```

## What's NOT in PR #13

- **The actual `dispatch_notification` worker job.** The repo has the
  primitives (`channels_for_kind`, `NotificationsRepo::create`); a real
  dispatch job would: subscribe to in-process events (purchase, course
  completion, refund, ...), look up the user's prefs, insert in-app +
  enqueue email per `channels_for_kind`. The worker hook lives in PR #15
  (refund) and a follow-up commit for course completion. Today, ad-hoc
  call sites (PR #15's refund handler) can call `NotificationsRepo::create`
  directly.
- **Email sender (Resend integration).** The `*_email` flag is recorded
  but no email leaves until the mailer crate ships. Tracked in the
  general PR #13 NOTES decision section.
- **DND-window enforcement.** The pref is stored; the dispatch job will
  honor it. Storage-side check is out of scope (multiple tz, the schema
  CHECK only validates structure not time logic).
