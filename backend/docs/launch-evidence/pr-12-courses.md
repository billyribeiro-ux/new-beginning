# PR #12 — Courses endpoints (player + progress + notes)

## Scope (BACKEND.md §11, §21 PR #12)

1. **`CourseProgressRepo`** — new storage repo. Three operations:
   - `mark_complete(enrollment, lesson_id, time_spent, payload)` — UPSERT
     on `(enrollment_id, lesson_id)`; time_spent **accumulates** on
     re-mark (extra viewing).
   - `upsert_notes(enrollment, lesson_id, notes)` — creates the row at
     zero progress if none exists; never overwrites time_spent.
   - `list_for_enrollment` + `count_completed` for read paths.
2. **`EnrollmentsRepo` extensions:**
   - `find_for_user_and_product` — entitlement-scoped lookup.
   - `update_progress(id, pct, last_lesson_id)` — recomputed by handler
     from `count_completed / total_lessons`. `completed_at` is stamped
     **exactly once** via a guarded CASE (idempotent).
3. **Four new authed routes:**
   - `GET /v1/courses` — active enrollments + product name/slug +
     progress_pct + last_lesson_id.
   - `GET /v1/courses/{slug}` — full player state: product, manifest
     (modules + lessons), per-lesson completion/notes, last cursor.
   - `POST /v1/courses/{slug}/progress` — `{lesson_id, time_spent_seconds, payload}`.
     Validates lesson_id against the manifest, marks complete, recomputes
     `progress_pct`. Returns the new pct.
   - `PUT /v1/courses/{slug}/lessons/{lesson_id}/notes` — `{notes}`.
     Upserts notes; does NOT flip completion.

## Why this shape

- **Manifest in `products.specs_json`** — keeps content authoring in the
  same admin surface as price/title/etc. PR #14 will add CRUD; for now
  the seeder populates `specs_json` from the catalog JSON.
- **Notes never clobbered by mark_complete** — typical user flow is
  take notes → click complete. UPSERT for completion sets time_spent +
  payload only; UPSERT for notes sets notes only. Tested with
  `mark_complete-after-notes` in the integration test.
- **`time_spent` accumulates on re-mark** — replay/re-watch is real
  signal; we lose nothing by adding to the prior total. The contrast
  pattern (last-write-wins) loses data when a user rewatches a lesson.
- **`completed_at` stamped via CASE WHEN $2 >= 100 AND completed_at IS
  NULL THEN now()** — idempotent. A repeat 100% update keeps the original
  completion timestamp. Tested explicitly.
- **`lesson_id` validated against the manifest** — refuses to store
  arbitrary text in `course_progress.lesson_id`; keeps the dataset on a
  closed vocabulary so admin reports (PR #14) and notifications (PR #13)
  can rely on the set of values.

## Anti-regressions 56–58

56. `mark_complete` MUST keep notes column out of the ON CONFLICT DO
    UPDATE clause. Dropping that omission would clobber a user's notes
    every time they hit "complete".
57. `update_progress`'s CASE WHEN guard on `completed_at` is
    load-bearing. Without `AND completed_at IS NULL`, every re-update at
    100% would refresh the completion timestamp, breaking certificate /
    achievement triggers that key off the first 100%.
58. `post_progress` MUST validate `lesson_id` against the manifest. A
    silent accept would open up the course_progress table to arbitrary
    `lesson_id` values, polluting analytics.

## Runtime evidence

`crates/http/tests/courses_integration.rs` walks the full storage path:
- Creates Alice + a course product with a 4-lesson manifest.
- Issues enrollment + license via the same tx-scoped path the webhook
  dispatcher uses.
- Marks `m1-l1` complete twice — verifies `time_spent_seconds`
  accumulates (60 + 30 = 90).
- Marks `m1-l2` complete — verifies `count_completed = 2` / 4 → 50%.
- Upserts notes on `m2-l1` without marking complete — verifies
  `count_completed` stays at 2 and the row carries the note text.
- Marks `m2-l1` complete — verifies notes survive (anti-regression #56).
- Marks `m2-l2` and updates to 100% — verifies `completed_at` stamps
  exactly once and a re-update to 100% does NOT move it
  (anti-regression #57).

```
$ DATABASE_URL=… cargo test -p tradeflex-http --test courses_integration
running 1 test
test courses_progress_and_notes_roundtrip ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Gates

```
$ cargo fmt --all                                           # clean
$ SQLX_OFFLINE=true cargo clippy --all-targets -- -D warnings  # clean
$ DATABASE_URL=… cargo test --workspace                     # 90+ tests pass
$ cargo deny check                                          # clean
```

## What's NOT in PR #12

- **Cert/achievement issuance on 100% completion.** The signal lives in
  `completed_at`; PR #13's notifications system will surface it. Cert PDFs
  ship in a later phase.
- **Real-time lesson sync** (e.g. video position broadcast). Out of
  scope; the `payload` JSONB on `course_progress` is there for it when
  needed.
- **Manifest validation as an admin-side check.** PR #14's product CRUD
  will validate the `specs_json` shape on save; today the handler just
  drops malformed JSON to an empty `modules: []` manifest, which presents
  as "no lessons" in the UI.
