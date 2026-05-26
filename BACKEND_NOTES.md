# TradeFlex Backend — Running Notes

A single append-only log of decisions, evidence, and surprises across every
PR in the BACKEND.md §21 rollout. This file is the cross-PR memory; per-PR
evidence still lives under `backend/docs/launch-evidence/`.

Each entry is dated and tagged with the PR it belongs to. New entries go at
the bottom of the relevant PR section.

---

## PR #1 — Workspace skeleton (2026-05-25) — ✅ shipped

**Scope (BACKEND.md §21 row 1):** workspace skeleton, `common::money`,
`AppError` + `AlertSink`, `Config`, `observability` crate, `/healthz`,
`/readyz`, CI green.

**Evidence:** [`backend/docs/launch-evidence/pr-01-skeleton.md`](backend/docs/launch-evidence/pr-01-skeleton.md).

### Gates

| Gate | Result |
|---|---|
| `cargo fmt --all -- --check` | ✅ |
| `cargo clippy --workspace --all-targets -- -D warnings` | ✅ |
| `cargo test --workspace` | ✅ 36/36 |
| `cargo deny check` | ✅ advisories / bans / licenses / sources all ok |
| `cargo run -p api` + curl /healthz, /readyz, /metrics | ✅ |

### Decisions made during PR #1

- **Toolchain:** Rust 1.95 (stable), edition 2021. `rust-toolchain.toml`
  pins channel and components (rustfmt, clippy).
- **`http` crate package name:** `tradeflex-http`, dep-aliased as `http-crate`
  in the workspace. Importing as `http_crate` in code. Reason: the bare name
  `http` collides with the `http` crate (HTTP types).
- **Middleware composition:** `axum::middleware::from_fn` does not compose
  cleanly inside a tower `ServiceBuilder`. Switched to stacked `Router::layer`
  calls (later `.layer()` calls become outer layers). Outside-in stack:
  CatchPanic → request_id → Trace → Timeout → BodyLimit → Compression →
  routes. Matches BACKEND.md §6 spec.
- **`TimeoutLayer::new` is deprecated** since tower-http 0.6.7. Using
  `TimeoutLayer::with_status_code(StatusCode::REQUEST_TIMEOUT, …)`.
- **Readiness pattern:** `ReadinessRegistry` holds `Arc<dyn ReadinessCheck>`s
  added via `AppState::with_readiness_check`. PR #1 ships zero checks; later
  PRs append (DB in PR #2, Stripe in PR #8). Empty registry → 200, which is
  the truthful answer for a no-deps build.
- **`AlertSink`:** declared as a first-class `AppState` field per BACKEND.md
  §6. Two impls today: `LogAlertSink` (prod default — emits at error level
  via `tracing`) and `RecordingSink` (test-only). Failing readiness checks
  fire `AlertKind::ReadinessDegraded`.
- **`figment::Jail` test helper** is gated behind the crate's `test` feature.
  Added as a dev-dep with that feature enabled. Its closure return type is
  `Result<(), figment::Error>` and `figment::Error` is 208 bytes — clippy's
  `result_large_err` fires. Since I don't own Jail's signature, I added
  `#[allow(clippy::result_large_err)]` on each `mod tests` that uses it.
- **`ConfigError::Figment`** wraps `Box<figment::Error>` for the same reason
  on the production path.
- **License field:** dropped from every workspace crate (`UNLICENSED` is not a
  valid SPDX identifier). cargo-deny config has `[licenses.private] ignore =
  true` to skip license checks for path-only workspace crates.
- **cargo-deny bans:** `allow-wildcard-paths = true` so workspace-internal
  `{ path = "..." }` deps don't trip the wildcard ban.

### MCP-tool fallback

The rust-analyzer MCP server is not surfaced in this session's tool list.
CLAUDE.md permits a fallback to `cargo check`/`cargo clippy` when the MCP is
unresponsive, called out explicitly — which we are doing for every PR until
the MCP server reappears. All edits ran through `cargo clippy
--all-targets -- -D warnings` before commit.

### Surprises / things to watch

- **Compile times:** first cold build was ~3 min on this Mac (aws-lc-sys is
  the long pole). Incremental rebuilds are <2 s. CI uses `Swatinem/rust-cache`
  to amortize.
- **`SetRequestIdLayer` from tower-http** was avoided in favor of a custom
  `axum::middleware::from_fn` because we need the request id to be visible
  in a `tokio::task_local` (so `AppError::into_response` can read it). The
  tower-http layer sets a header but doesn't bind a task-local.

### Anti-regressions (carry forward forever)

1. `Money` has no `Mul` impl for `i64` or `Money`. Every multiplication is
   `mul_qty` / `mul_bps` / `mul_pct` returning `Result`.
2. `AppError::Internal` never echoes the source string to the response body.
3. `AlertSink` is the only mechanism for "should-never-happen" reporting.
4. `/healthz` ignores all dependencies. `/readyz` reflects them honestly via
   the registry.
5. Request-id is generated when absent and propagated when present, on every
   route.
6. JSON log output is structured (one event per line, valid JSON).

---

## PR #2 — Migrations + storage::pool + seeder + dump-catalog (2026-05-25) — ✅ shipped

**Scope (BACKEND.md §21 row 2):** Migrations 0001–0021; `storage::pool`;
`seeder` binary; catalog snapshot dump script.

**Evidence:** [`backend/docs/launch-evidence/pr-02-migrations-storage-seeder.md`](backend/docs/launch-evidence/pr-02-migrations-storage-seeder.md).

### Gates

| Gate | Result |
|---|---|
| `cargo fmt --all -- --check` | ✅ |
| `SQLX_OFFLINE=true cargo clippy --workspace --all-targets -- -D warnings` | ✅ |
| `cargo sqlx prepare --workspace --check` | ✅ |
| `cargo test --workspace` | ✅ 36/36 |
| `cargo deny check` | ✅ |
| `just up && just migrate && just seed` | ✅ |
| `/readyz` toggles 200 ↔ 503 with DB | ✅ |

### Decisions made during PR #2

- **Host port for dev Postgres:** **5435**, not 5432. The default port was
  already taken by a system-installed Postgres + a sibling project's
  container. The compose file reads `PG_HOST_PORT` env var (default 5435)
  to let other machines override.
- **`bigdecimal` added to sqlx features.** `products.rating_value` is
  `NUMERIC(3,2)` per BACKEND.md §4; sqlx requires either `bigdecimal` or
  `rust_decimal` to bind it. Chose `bigdecimal` because it's already
  pulled in transitively and the API is straightforward.
  In the seeder we format f64 → string with `{:.2}` → `BigDecimal::from_str`
  to enforce the scale at the application boundary.
- **`webpki-roots` license `CDLA-Permissive-2.0`** added to
  `deny.toml`'s allow-list. Permissive (no copyleft); covers the bundled
  Mozilla CA store used by rustls.
- **Migrations 0001–0021 are the complete set.** BACKEND.md §1.6 / §4 say
  "no 0022"; I respected that. Migration 0022 mentioned earlier (the
  `rate_limit_counters` sketch) is intentionally omitted — it's a Phase 3
  forward path, not part of the v1 schema.
- **`ReadinessCheck` trait moved from `http` to `common`.** `storage` needs
  to impl it without depending on axum. Same trait, same registry, just
  rehoused. The http crate re-exports `common::readiness` so existing
  call-sites are unaffected.
- **DB readiness check timeout: 100 ms.** Matches BACKEND.md §15 spec.
  Wraps `SELECT 1` in a `tokio::time::timeout`; on miss, emits
  `"fail: timeout after 100ms"`.
- **`dump-catalog` is a drift-checker, not an auto-extractor.** The
  frontend `src/lib/data/products.ts` imports Tabler icon components via
  subpaths that Node's `tsx` cannot resolve (their `package.json` exports
  map blocks them). Auto-extraction would require either (a) splitting
  data-only constants into a separate file the frontend re-imports, or
  (b) bundling with esbuild + a stub plugin. Neither is worth doing in
  PR #2 with only 5 catalog rows. The hand-written `seeds/catalog.json`
  is reviewed in PRs and `pnpm run dump-catalog` regex-parses the TS
  sources to assert every `(slug, priceCents)` pair matches.
  **Follow-up:** when the catalog grows past ~20 rows, refactor the
  frontend data layer or add a bundler step here.
- **`PoolError::MissingUrl`** instead of making `DATABASE_URL` required
  at the config layer. Keeps tests free to construct a Config without a
  DB; the api binary calls `storage::build_pool` which surfaces the error
  at boot if the URL is missing.
- **`.sqlx/` cache is committed.** CI runs with `SQLX_OFFLINE=true` so the
  cache is authoritative; the `sqlx-prepare-check` CI job spins up its
  own Postgres + runs `cargo sqlx prepare --check` to fail on drift.

### Surprises / things to watch

- **`SET TRANSACTION ISOLATION LEVEL REPEATABLE READ`** is mentioned in
  BACKEND.md §9.5 for the data-export job. Not implemented in PR #2 —
  the export job is PR #16. Noting here so the future implementer
  doesn't forget that step.
- **`enrollments.unique(user_id, product_id)` collides with PR #16's
  manual-grant flow** if a user has a purchase-source enrollment, then
  gets a manual-source upgrade. BACKEND.md §8.5 says "purchase enrollments
  are forever" so the constraint is correct for v1 — manual grants
  upsert into the existing row rather than insert a second.
- **Seeder upserts use `Uuid::now_v7()` for new IDs but ignore the value
  on conflict** — meaning a re-run never changes existing ids. Documented
  behavior; matches BACKEND.md §1.6.
- **Docker compose collisions:** between the local stack, another
  project's `rtp-db` (postgres on 5432), and a system Postgres, 5432 and
  5434 are both already taken on this workstation. Chose 5435.

### Anti-regressions (added in PR #2; carry forward)

7. `.sqlx/` cache must be committed and in sync. CI `sqlx-prepare-check`
   gates this.
8. Catalog snapshot drift between `src/lib/data/*.ts` and
   `backend/seeds/catalog.json` is caught by `pnpm run dump-catalog`.
9. Migrations are append-only. Editing or renaming a shipped file is
   forbidden.
10. Money fields in NUMERIC columns route through `BigDecimal`, never
    `f64` past the boundary.

---

## PR #3 — Auth (signup/login/logout) + sessions + Argon2id + service_token (2026-05-25) — ✅ shipped

**Evidence:** [`backend/docs/launch-evidence/pr-03-auth.md`](backend/docs/launch-evidence/pr-03-auth.md).

### Gates

| Gate | Result |
|---|---|
| fmt / clippy `-D warnings` / sqlx-prepare-check | ✅ |
| `cargo test --workspace` | ✅ 46/46 |
| `cargo deny check` | ✅ |
| signup → 201 + DB row + `HttpOnly` cookie | ✅ |
| password verify round-trip across process restart | ✅ |
| login lockout fires 429 with `Retry-After` | ✅ |
| service_token rejects missing / wrong / unknown token | ✅ |

### Surprises that ate time

- **`async_trait` removed.** Axum 0.8's `FromRequestParts` uses native
  async-fn-in-traits; slapping `#[async_trait]` on an impl produces a
  lifetime-mismatch error that's not obvious from the spec. Lesson: when
  the trait declaration uses `impl Future`, drop the macro.
- **`governor` 0.7's `wait_time_from`** takes `QuantaInstant`, not
  `std::time::Instant`. Solution: hold a `DefaultClock` next to the limiter
  and pass `clock.now()`.
- **`figment::Jail` test gating.** Already documented in PR #1, but bit us
  again in `routes/health.rs` when expanding `AppState::new` — the tests
  needed a live DB to construct repos. The pragmatic fix was to plumb a
  real `PgPool` against the dev compose stack and require `DATABASE_URL`
  in tests. CI already exports it; local devs need `just up` first.
- **`http::header` import in `signup.rs`** had to use `axum::http::header`,
  not the standalone `http` crate that's also a dep (axum re-exports it
  but the standalone name shadows). Renamed the import to keep both happy.
  (See `routes/auth/signup.rs`.)
- **`pgcrypto` is still installed but unused so far.** Per BACKEND.md
  §1.6 we keep it for future operational queries. PR #3 doesn't touch it.
- **Cookie `Secure` flag is `env`-gated.** Production (`env=production`)
  sets it; dev does not, because `SameSite=Lax + Secure` would prevent
  the cookie from being set on `localhost` without HTTPS. Documented at
  the call site (`signup::format_cookie`). Re-evaluate when staging is on
  HTTPS.

### Decisions deferred (intentional, to avoid scope creep)

- **`last_seen_at` 60 s coalescing** (BACKEND.md §7.1) → not in PR #3;
  every load currently writes. Profile first, then add `moka` only if
  the write rate justifies it.
- **Constant-ish-time path on unknown email** → not in PR #3. A future
  bag of tests will measure timing and decide whether to "always
  verify against a known-bad hash" is worth the cycles.
- **Capturing `User-Agent` on session create** → not in PR #3 because
  there's no `/v1/me/sessions` UI yet to surface it; user_agent column
  stays NULL until PR #4.
- **The `revoke_all_except` flow** → method shipped in `SessionsRepo`,
  no endpoint wired yet (PR #4's `/v1/me/sessions/revoke-others`).

### Anti-regressions (added in PR #3; carry forward)

11. Argon2id pepper is ALWAYS the keyed `secret`, never concatenated.
    `hash_password` + `verify_password` live in the same module and
    share `argon2_with_pepper()` to lock the symmetry.
12. Session cookie HMAC is verified constant-time BEFORE any DB lookup.
13. `service_token` is required at api boot; missing → bail.
14. `LoginLimiter` runs BEFORE Argon2 verify on the login path. An
    attacker cannot make us burn CPU on Argon2 without first being inside
    the bucket.
15. `UsersRepo::create` returns `DuplicateEmail` distinctly; the BFF can
    surface 409 to the user without an internal-error leak.
16. Sessions are revocable from the server side — the cookie alone is
    not a bearer token.

---

## PR #3 — original scope notes (kept for audit)

The original "in progress" block lived here. Resolution above.

---

## PR #4 — `/v1/me/*` (profile + sessions + change-email + change-password) (2026-05-25) — ✅ shipped

**Evidence:** [`backend/docs/launch-evidence/pr-04-me.md`](backend/docs/launch-evidence/pr-04-me.md).

### Gates

| Gate | Result |
|---|---|
| fmt / clippy `-D warnings` / sqlx-prepare-check | ✅ |
| `cargo test --workspace` | ✅ 48/48 |
| `cargo deny check` | ✅ |
| signup → GET /v1/me round-trip | ✅ |
| PATCH tri-state headline (set / null-clear / absent-no-op) | ✅ |
| GET /v1/me/sessions + `current: true` flag for caller's session | ✅ |
| revoke-others revokes peers, keeps caller | ✅ |
| change-password rejects wrong-current, revokes other sessions, login with new works, login with old rejected | ✅ |
| change-email stages email_verifications row + EMAIL_STUB log line | ✅ |

### Surprises that ate time

- **Serde tri-state.** `Option<Option<String>>` does NOT round-trip through
  serde with default behavior — absent and `null` both deserialize to
  `None`. The fix is a `double_option` helper:
  ```rust
  fn double_option<'de, T, D>(d: D) -> Result<Option<Option<T>>, D::Error>
  where T: Deserialize<'de>, D: Deserializer<'de>
  { Deserialize::deserialize(d).map(Some) }
  ```
  + `#[serde(default, deserialize_with = "double_option")]`.
  Discovered during PR #4 runtime evidence (first PATCH with `"headline":
  null` did not clear the column). Cost: ~10 minutes including writing
  the test cases. Encoded as anti-regression #17.

- **`UPDATE … CASE WHEN $3::boolean THEN NULL ...`.** Postgres needed
  explicit casts on the boolean and text params so sqlx's compile-time
  type inference resolved them. The CASE expression around a tri-state
  is verbose but it kept the whole UPDATE in a single statement — adding
  multiple branches with multiple UPDATEs would have been worse.

- **`citext` cast on `email_verifications.new_email`.** Same trick as
  `users.email` — pass `Option<&str>` to a `$5::citext` param. sqlx
  resolved cleanly once the `.sqlx/` cache was regenerated.

### Decisions deferred (intentional)

- **Real mailer.** `change-email` logs the verification token instead of
  sending it. The mailer crate (BACKEND.md §10) is implicit in the §21
  rollout; it'll land alongside or just before PR #7 when receipts need
  to fire. Until then, the verify-email endpoint isn't wired and a user
  can't actually complete an email change — but the staged row is
  correct.
- **Geo-IP enrichment on `/v1/me/sessions`.** Just shows the raw IP for
  now; PR #14's admin/customers view will need a richer location lookup
  and we'll consolidate.
- **User-agent capture.** PR #3 left `user_agent` NULL on session create.
  PR #4 didn't wire it either — that's a 2-line change once we add a
  `TypedHeader<UserAgent>` extractor on login/signup; deferring to
  bundle with PR #5's TOTP flow which also extends `SessionsRepo`.

### Anti-regressions (added in PR #4; carry forward)

17. PATCH bodies with optional-clearable fields use `double_option`
    deserializer. Default `Option<Option<T>>` is a bug.
18. Password change requires verify-current AND revokes peer sessions
    (current session stays alive so the user isn't logged out by their
    own action).
19. `/v1/me/*` endpoints all gated by `require_auth`. No public surface
    leak.
20. Verification tokens (email-change, future password-reset / signup-
    verify) are stored as `sha256(plaintext)`; plaintext lives only
    in the link the mailer sends.

---

## PR #5 — TOTP enable/confirm/disable + backup codes (2026-05-25) — ✅ shipped

**Evidence:** [`backend/docs/launch-evidence/pr-05-totp.md`](backend/docs/launch-evidence/pr-05-totp.md).

### Gates

| Gate | Result |
|---|---|
| fmt / clippy `-D warnings` / sqlx-prepare-check | ✅ |
| `cargo test --workspace` | ✅ **70/70** (24 new from PR #5) |
| `cargo deny check` | ✅ |
| Every 2FA bullet from BACKEND.md §16.3 | ✅ |

### Surprises that ate time

- **`secrecy` 0.10 removed `SecretVec`** in favor of generic `SecretBox<T>`.
  Used `pub type SecretBytes = SecretBox<Vec<u8>>;` in
  `auth/totp_secret_at_rest.rs` so the rest of the codebase reads naturally.
- **`qrcode` 0.14 dropped the `png` feature** (`svg` is the only remaining
  one). Dropped the standalone crate entirely — `totp-rs` already ships a
  `qr` feature that returns a base64-PNG. One fewer dep.
- **`totp-rs::get_url()` omits default fields** (`algorithm`, `digits`,
  `period`). Google Authenticator and clones interpret missing fields as
  SHA-1 / 6 / 30, so the URI is still correct — but the test had to
  relax its assertions about the fields and just assert issuer + account
  + secret presence.
- **No `oathtool` or `pyotp`** on the local box. Built a 13-line
  `/tmp/totp_cli.py` (RFC 6238 with stdlib `hmac` + `base64`) and used it
  to generate codes for runtime evidence — `skew = -30` for the previous
  step (must pass), `-60` for two-steps-ago (must fail).
- **The login-attempt limiter applies to both stages.** Initial runtime
  evidence kept tripping 429 because each evidence step burned 1–2
  bucket entries; ended up sprinkling `sleep 65` between scenarios.
  Documented in the evidence file as the price of the limiter being
  load-bearing.

### Decisions deferred (intentional)

- **`tx_id` single-use replay protection.** Currently the HMAC-signed
  pending-TOTP token is valid for 5 minutes with no consumption table.
  An attacker who steals BOTH the tx_id AND a valid 30-90 s TOTP code
  within that window could reuse the pair. Mitigations already in place:
  (a) limiter throttles 5/min, (b) the TOTP code itself is short-lived,
  (c) `tx_id` TTL is 5 min. Phase-2 work: add a `consumed_pending_totp`
  bloom filter or a `pending_totp_uses(tx_id_hash, used_at)` table.
- **TOTP-secret key rotation** for `AUTH_TOTP_KEY`. If the key changes,
  every encrypted secret in the DB becomes garbage. Phase 3: prefix the
  ciphertext with a key-version byte and support decrypt-old +
  re-encrypt-new on the next confirm.
- **`PR #5 didn't wire user-agent into the session row** even though
  PR #4 noted it. Postponing again to PR #6+ when the public surface
  refactor exposes the `TypedHeader<UserAgent>` axum extractor on more
  routes.

### Anti-regressions (added in PR #5; carry forward)

21. TOTP params (SHA-1, 6 digits, **skew=1**, 30s period) are literals in
    the constructor — never crate defaults.
22. TOTP secrets at rest = `XChaCha20-Poly1305([nonce:24][ct+tag])` under
    `AUTH_TOTP_KEY`. Tamper / wrong-key surfaces as `Err(Decrypt)`,
    never silent garbage.
23. Backup codes use Crockford base32 (no `I`/`L`/`O`/`U`/`0`-as-O) and
    are Argon2id-hashed via the same `argon2_with_pepper` builder as
    passwords.
24. Backup-code consumption is a CAS on the pre-image bytes;
    `consume_backup_code_slot` returns `false` if the slot moved —
    concurrent uses cannot double-redeem.
25. `POST /v1/me/2fa/disable` requires BOTH current password AND current
    code. A shoulder-surfed password cannot drop 2FA.
26. Pending-TOTP token HMAC includes a domain separator
    (`tfx_pending_totp.`) so a session cookie minted with the same key
    cannot be replayed as a tx_id (and vice versa).
27. `AUTH_TOTP_KEY` is required at api boot — missing → bail.
28. `LoginLimiter::check` is consulted at both `/v1/auth/login` AND
    `/v1/auth/login/totp` so passing stage 1 doesn't open unlimited
    stage-2 attempts.

---

## PR #6 — public surface (products/plans/leads/contact) + general rate limiter + BFF cutover (2026-05-25) — ✅ shipped

**Evidence:** [`backend/docs/launch-evidence/pr-06-public-surface.md`](backend/docs/launch-evidence/pr-06-public-surface.md).

### Gates

| Gate | Result |
|---|---|
| fmt / clippy `-D warnings` / sqlx-prepare-check | ✅ |
| `cargo test --workspace` | ✅ **74/74** |
| `cargo deny check` | ✅ |
| `pnpm run check` (svelte-check) | ✅ 0 errors / 0 warnings |
| `EXPLAIN ANALYZE` shows `Index Scan using leads_created_at_idx` | ✅ |
| Lead capture lockout fires at 6th attempt | ✅ |
| Contact lockout fires at 4th attempt | ✅ |

### Surprises that ate time

- **`base64-no-pad` of a 16-byte UUID has 4 fake trailing bits.**
  `auth::pending_totp_token::rejects_tampered_user_id` failed because
  flipping the last char of the b64 segment could yield bit-identical
  bytes after decode. Tampering a middle char (5th position) fixes it.
  Encoded as a test-side fix in PR #6.
- **`zsh` brace expansion ate my python-inlined `f-strings`** in the
  evidence script (`{cents/100:.2f}` got parsed as a math expression).
  Worked around by splitting the script into `/tmp/dump_products.py`
  + `/tmp/dump_plans.py`. Not a code issue, but worth noting for the
  next operator running evidence by hand on macOS.
- **`Vec<String>` columns** in `products.highlights/deliverables/
  requirements` deserialize cleanly through sqlx's `pg-array` support
  with no extra opt-in. The `JSONB` `specs_json` rides as
  `serde_json::Value` — both shapes are exactly what the BFF wants.

### Decisions deferred (intentional)

- **Honeypot still consumes a rate-limit token** because the limiter
  runs BEFORE the honeypot check. A real bot abusing the form will hit
  429 either way; a one-off bot probe gets a silent 201 plus a wasted
  token. Worth optimizing only if false-positive honeypot trips become
  a noisy signal in prod.
- **`PR #14 admin/leads /admin/messages` BFF cutover** stays in PR #14
  per BACKEND.md §21. PR #6 only rewrote the two BFF actions BACKEND.md
  §21 row 6 specified.
- **Trusted-proxy support for `X-Forwarded-For`.** The BFF helper
  forwards the client IP via `x-forwarded-for`, but Rust currently
  reads the peer address from `ConnectInfo<SocketAddr>` (the TCP
  socket) — which in the BFF→Rust hop is the BFF's IP, not the
  end-user's. PR #17 (cutover) adds a `RealIp` middleware that trusts
  XFF when the source is in `SERVICE_TOKEN_IP_ALLOWLIST`. Until then,
  the per-IP buckets are keyed by BFF IP in production; in local dev
  the BFF runs on the same loopback so it Just Works.

### Anti-regressions (added in PR #6; carry forward)

29. `RateLimiterSet` is the ONE rate-limit primitive — handlers pick a
    `Bucket`, never construct their own limiter.
30. Honeypot path on lead/contact: silent 201, no DB write.
31. JSONB / array columns surface through the API verbatim; the BFF
    owns UI shaping.
32. Cents stay `i64` end-to-end on the wire; `Money` serde guard
    rejects out-of-JS-range values (anti-regression #1, still
    load-bearing).
33. BFF → Rust calls go through `callRust()` only. Centralizes
    header injection, error mapping, request-id propagation.

---

## PR #7 — stripe-client + /v1/checkout + webhook receiver (2026-05-25) — ✅ shipped

**Evidence:** [`backend/docs/launch-evidence/pr-07-stripe-skeleton.md`](backend/docs/launch-evidence/pr-07-stripe-skeleton.md).

### Gates

| Gate | Result |
|---|---|
| fmt / clippy `-D warnings` / sqlx-prepare-check | ✅ |
| `cargo test --workspace` | ✅ **83/83** (+9 new) |
| `cargo deny check` | ✅ |
| Webhook sig: missing / wrong-key / stale / valid → 400/400/400/200 | ✅ |
| Webhook replay deduped | ✅ |
| Crash-recovery: stranded row + reconciliation query + re-POST → processed | ✅ |

### Surprises that ate time

- **`async-stripe` is brittle.** PR #3's BACKEND.md notes warned this
  would happen; concrete failure: the `webhook-events` feature on
  versions 0.40/0.41 won't compile against `checkout` + `billing`
  without also enabling `connect` (which we'll never use). Wrote our
  own thin `reqwest` wrapper instead. Saved hours of feature-gate
  whack-a-mole and pulled the swap-out cost of BACKEND.md §23 risk #1
  to today.
- **`AppState` needed a raw `PgPool` field** for the webhook handler's
  ad-hoc tx. Adding `pub db: PgPool` to `AppState` is cheap (it's
  internally `Arc`) and turned the handler from a tangled
  manual-rollback into a clean `state.db.begin()`.
- **Postgres `xmax = 0` trick** is the canonical way to distinguish
  fresh-insert vs. recognized-existing in an `ON CONFLICT DO UPDATE`
  RETURNING. Less googleable than it deserves — pin in the repo doc
  comments so the next implementer doesn't re-derive it.
- **`validator` 0.20 requires every field of a struct deriving
  `Validate` to implement `Serialize`** (it embeds field values into
  error metadata). Adding `Serialize` to `CheckoutRequest` +
  `CheckoutLineInput` solved it but is the kind of thing that's
  easy to miss in a new validation-bearing type.
- **clippy `cast-abs-to-unsigned`** vs. `(x - y).abs() as u64` →
  `(x - y).unsigned_abs()`. Same value, type-system-honest.

### Decisions deferred (intentional)

- **Real Stripe test keys.** Without them, `/v1/checkout` can't actually
  call Stripe end-to-end. PR #7 verifies the call path compiles + the
  request shape via `RecordingStripeApi`; real checkout happens in
  PR #8 when handler-side asserts kick in.
- **Reconciliation cron.** Lives in the `worker` binary (PR #11). PR #7
  ships `StripeEventsRepo::list_unprocessed_older_than` so the cron is
  a 5-line addition.
- **`products.stripe_price_id` wiring.** Products currently return a
  400 from `/v1/checkout` because the seeder doesn't populate
  `stripe_price_id`. PR #8 adds a per-product Stripe Price sync (or a
  small admin task that writes it in once).
- **BFF `/api/webhooks/stripe` forwarder.** The Rust receiver currently
  expects the BFF to pass the raw body + `stripe-signature` header
  through unchanged AND inject `x-service-token`. The BFF route lands
  in PR #20 (cutover); for evidence we hit the Rust receiver directly
  with the service token (no public Rust URL — same as every other
  PR).

### Anti-regressions (added in PR #7; carry forward)

34. Stripe writes ALWAYS carry an `Idempotency-Key` header from
    `common::auth::idempotency::derive_key`. Retries collapse.
35. Webhook signature: HMAC-SHA256 over `"{t}.{raw_body}"`,
    constant-time compare via `subtle`, 5-min staleness tolerance.
36. The webhook receiver returns 200 unless sig invalid or DB claim
    fails. Dispatcher errors fire an `AlertSink::WebhookHandlerFailed`
    + leave `processed_at = NULL` for the reconciliation cron.
37. `stripe_events.claim` is the ONE primitive for idempotency. No
    handler invents its own `IF EXISTS` check.
38. `dispatch()` and `mark_processed_in_tx` ALWAYS run in the same
    Postgres tx. A crash leaves `processed_at = NULL` — the load-
    bearing invariant the reconciliation cron depends on.
39. Wire-level Stripe client = thin `reqwest` wrapper, not
    `async-stripe`. Swap cost stays low.

---

## PR #8 — checkout.session.completed handler + entitlements + reconcile_stripe_events (2026-05-25) — ✅ shipped

**Evidence:** [`backend/docs/launch-evidence/pr-08-entitlements.md`](backend/docs/launch-evidence/pr-08-entitlements.md).

### Gates

| Gate | Result |
|---|---|
| fmt / clippy `-D warnings` / sqlx-prepare-check | ✅ |
| `cargo test --workspace` | ✅ **86/86** |
| `cargo deny check` | ✅ |
| Indicator / course / subscription dispatch paths all green | ✅ |
| Replay dedupe verified (license count unchanged) | ✅ |
| GET /v1/subscription returns the active mirror | ✅ |

### Decisions deferred (intentional)

- **Real Stripe Price ids** still placeholder (`price_test_TF_{slug}`).
  Wired into the seeder; PR #14's admin product CRUD or a one-off ops
  script overwrites with real prices.
- **`invoice.paid` and `customer.subscription.*`** are PR #9. Today
  unknown event kinds log + return Ok (so the dedupe row stamps
  `processed_at` rather than re-firing).
- **`reconcile_stripe_events` cron schedule.** Function is callable; the
  worker binary (PR #11) wires it on a 5-min cron.

### Anti-regressions (40–44 in evidence doc)

---

## PR #17 — cutover: flag flip + production env wiring + Drizzle deletion plan (2026-05-25) — ✅ shipped

**Evidence:** [`backend/docs/launch-evidence/pr-17-cutover.md`](backend/docs/launch-evidence/pr-17-cutover.md).

### What this PR does

| Item | Notes |
|---|---|
| `useRustBackend()` flip | `=== 'true'` → `!== 'false'`. Rust is now the default; setting `USE_RUST_BACKEND=false` is the explicit fallback. |
| Production env-vars runbook | Every var the api + worker + BFF need, with notes on which are fail-closed-in-prod. |
| Drizzle deletion plan | Enumerated, ordered, per-file. Deletion deferred to a separate follow-up PR after a 7-day rollback window. |

### Anti-regressions 72–73

72. The flag flip is **opt-out** (`!== 'false'`) so a missing env defaults Rust ON. Reverting requires explicit `USE_RUST_BACKEND=false`.
73. Drizzle file deletion is deferred to a separate PR — mixing the flag flip with file removal eliminates the rollback path.

### Decisions

- **Rollback window of 7 days** before Drizzle is hard-deleted.
- **No silent prod fallback** for missing secrets — api + worker bail at boot.
- **BFF stays on Vercel; Rust stays on Railway.** Webhook endpoint at the BFF (`/api/webhooks/stripe`), service-token-gated; no public Rust URL.

### Original scope notes (for audit)

**Scope (BACKEND.md §21 row 17):** Drizzle/SQLite removal + production env wiring + Vercel BFF env vars. The flag flip + runbook ship today; the file removals land after the rollback window per BACKEND.md §22 rule 12 (no destructive ops without explicit need).

---

## PR #16 — data export + account deletion (2026-05-25) — ✅ shipped

**Evidence:** [`backend/docs/launch-evidence/pr-16-export-deletion.md`](backend/docs/launch-evidence/pr-16-export-deletion.md).

### Surfaces

| Surface | Notes |
|---|---|
| `GET /v1/me/export` | Inline JSON dump (user/orders/invoices/licenses/enrollments/sub/notifications). |
| `DELETE /v1/me` | Soft-delete; pseudonymizes email, clears creds, purges sessions, audits `account.deleted`. Idempotent. |
| `UsersRepo::soft_delete` | Same-tx user update + session DELETE. |

### Anti-regressions 70–71

70. Session purge MUST share the tx with the user row update — anti stolen-cookie.
71. `find_by_email/by_id` MUST keep `deleted_at IS NULL`.

### Decisions

- **Soft, not hard-delete** — orders/invoices/audit retained per legal requirement. Hard-purge is a follow-up retention job.
- **Pseudonymized email instead of NULL** — column is NOT NULL + UNIQUE; pseudonym satisfies both.
- **Inline JSON export** — small footprints today; async-to-R2 lands when largest user crosses ~5 MB.

---

## PR #15 — refund flow (admin trigger + charge.refunded handler + revoke + notification) (2026-05-25) — ✅ shipped

**Evidence:** [`backend/docs/launch-evidence/pr-15-refund.md`](backend/docs/launch-evidence/pr-15-refund.md).

### Surfaces added

| Surface | Notes |
|---|---|
| `POST /v1/admin/orders/{id}/refund` | Admin trigger; idem-keyed on `(env, actor, "order_refund", order_id)`; audits `admin.refund.triggered`. NO inline revoke. |
| `charge.refunded` dispatcher | Single source of truth. One tx: mark_refunded → revoke licenses + enrollments → audit `order.refunded`. Notification outside tx. |
| `StripeApi::refund_payment_intent` | New trait method; `StripeRefund` struct; recording fake. |
| `OrdersRepo::mark_refunded_in_tx` | Idempotent (status='paid' gate). |
| `LicensesRepo::revoke_for_order_in_tx` | Idempotent (active=TRUE + source='purchase' gate). |
| `EnrollmentsRepo::revoke_for_order_in_tx` | Same gate. Sub-source enrollments survive a one-shot refund. |

### Anti-regressions 67–69

67. `mark_refunded_in_tx`'s `status = 'paid'` predicate is load-bearing for idempotency.
68. License/enrollment revoke MUST key off both `source_ref_id = order_id` AND `source = 'purchase'` so sub-source rows aren't dragged down by a one-shot refund.
69. Refund Idempotency-Key MUST include `order_id` — otherwise different orders collapse to the same Stripe call.

### Decisions

- **No inline revoke at refund-trigger time.** Source of truth is `charge.refunded`. Crash-safety via existing `stripe_events` replay; reconcile cron re-fetches dropped POSTs.
- **Notification outside the tx.** Re-drive can dupe; lesser evil vs. losing audit consistency.
- **Sub-invoice refunds NOT handled here.** Subscription cancellation flows through `customer.subscription.deleted` (PR #9).

---

## PR #14 — admin endpoints (KPIs + leads/messages + products active toggle + customers search + manual grant) (2026-05-25) — ✅ shipped

**Evidence:** [`backend/docs/launch-evidence/pr-14-admin.md`](backend/docs/launch-evidence/pr-14-admin.md).

### Surfaces added

| Route | Notes |
|---|---|
| `GET /v1/admin/stats` | revenue_30d_cents, orders_30d, active_subscriptions, signups_30d, leads_30d. |
| `GET /v1/admin/leads` | Newest first, default limit 200, max 1000. |
| `GET /v1/admin/messages?status=` | Closed status set: `new/read/archived/spam`. |
| `PATCH /v1/admin/messages/{id}` | Status change; audit row in same tx. 404 + rollback on 0 rows. |
| `GET /v1/admin/products?kind=` | Includes inactive (admin view). |
| `PATCH /v1/admin/products/{id}` | `active` toggle; audit action asymmetric (`admin.product.activated` / `admin.product.deactivated`). |
| `GET /v1/admin/customers?q=&limit=50` | `ILIKE %q%` on citext email. |
| `POST /v1/admin/customers/{id}/grant-entitlement` | License + (course) enrollment + audit row in one tx. Requires `reason` (4–500 chars). |

Storage additions: `LeadsRepo::list/count_since`, `ContactRepo::list/update_status`, `OrdersRepo::list_recent/revenue_since`, `UsersRepo::count_since/search_by_email`, `SubscriptionsRepo::count_active`, `ProductsRepo::list_all/set_active`.

### Anti-regressions 63–66

63. `require_admin` lives on the admin `Router::route_layer`, separate from the `authed` tree.
64. `customers::grant_entitlement` synthesizes a fresh `OrderId` per grant — the licenses-repo skip check keys off `(user, product, source_ref_id)`, so a reused id would silently no-op while the audit row claimed a fresh license.
65. `messages::patch` MUST rollback if UPDATE returns 0 rows. Otherwise the audit row claims a status change that never happened.
66. Product PATCH audits both directions distinctly (`activated` / `deactivated`) — keeps alert/grep rules trivial; a single `toggled` action would hide the new state in JSON.

### Decisions

- **`active_subscriptions` is a count, not MRR cents.** Real MRR needs `subscription_plans.price_cents` populated; today those are placeholders from the seeder. Recorded as a known follow-up.
- **Full product/plan CRUD is deferred** — `active` toggle is the only mutation today. There's no admin UI consumer for the larger CRUD yet; ship it when the editor pane lands.
- **Manual grant `reason` is required** (4–500 chars). Every "why did this account get this entitlement?" question is answerable from `audit_log`.
- **Customer search is `ILIKE %q%`** on citext column — small data set; trigram index lands when the user table hits ~100k.

### Original scope notes (for audit)

**Scope (BACKEND.md §21 row 14):** admin KPIs + leads/messages + products/plans CRUD + orders/customers. Full CRUD + order detail are scoped down to "what the dashboard actually consumes today"; the rest is deferred to a follow-up that lands alongside the admin UI editor pane.

---

## PR #13 — notifications repo + feed + preferences + channels_for_kind (2026-05-25) — ✅ shipped

**Evidence:** [`backend/docs/launch-evidence/pr-13-notifications.md`](backend/docs/launch-evidence/pr-13-notifications.md).

### Surfaces added

| Surface | Notes |
|---|---|
| `NotificationsRepo` | `create`, `list_for_user(unread, limit)`, `mark_read` (no-op on wrong owner), `mark_all_read`, `unread_count`. |
| `NotificationPreferencesRepo` | `get_or_default` (lazy materialization), `patch` (partial update with double-Option DND times). |
| `channels_for_kind` | Pure fn mapping kind slug → `(email_opt, inapp_opt)`. Unknown kinds → in-app only. |
| `common::deserialize::double_option` | Lifted from `me/profile.rs` for reuse. |
| `GET /v1/notifications` | Paginated feed (default 50, max 200). |
| `PATCH /v1/notifications/{id}/read` | Silent no-op on wrong owner — anti-enumeration. |
| `POST /v1/notifications/mark-all-read` | Returns `{updated: n}`. |
| `GET/PATCH /v1/notifications/preferences` | Lazy materialization on first GET. PATCH validates DND triple pre-DB. |

### Anti-regressions 59–62

59. `mark_read` MUST 204 on wrong-owner. 404 would let an attacker enumerate IDs.
60. `get_or_default` is `INSERT … ON CONFLICT DO NOTHING RETURNING` + fallback fetch — atomic. SELECT-then-INSERT would race.
61. `channels_for_kind`'s unknown-kind fallback is `(false, true)` — in-app only, never silently drop, never spam email.
62. PATCH-prefs MUST validate the FINAL state (current row + patch) for DND triple, not just the patch fields. A partial patch could otherwise flip `dnd_enabled=true` without ever sending the times.

### Decisions

- **Lazy materialization** over write-at-signup — most users never visit the page.
- **`mark_read` silent on wrong owner** — anti-enumeration, more important than the convenience of distinguishing 200/404.
- **`channels_for_kind` as a pure fn** — exhaustively unit-testable; new kinds are a match arm.
- **DND tri-state via `Option<Option<Time>>`** — needed to distinguish "leave the time alone" from "clear it".

### Original scope notes (for audit)

**Scope (BACKEND.md §21 row 13):** notifications (prefs + feed) + dispatch_notification job. The repo + endpoints ship in this PR; the dispatch worker job ships incrementally as call sites emit events (PR #15 refund first).

---

## PR #12 — courses endpoints + course_progress repo + enrollment progress recompute (2026-05-25) — ✅ shipped

**Evidence:** [`backend/docs/launch-evidence/pr-12-courses.md`](backend/docs/launch-evidence/pr-12-courses.md).

### Surfaces added

| Surface | Notes |
|---|---|
| `CourseProgressRepo` | `mark_complete` (UPSERT, time_spent **accumulates**), `upsert_notes` (UPSERT notes only), `list_for_enrollment`, `count_completed`. |
| `EnrollmentsRepo::find_for_user_and_product` | Entitlement-scoped lookup. |
| `EnrollmentsRepo::update_progress` | Recomputes `progress_pct` + `last_lesson_id`; `completed_at` stamped **exactly once** via CASE guard. |
| `GET /v1/courses` | List active enrollments + product slug/name + progress_pct. |
| `GET /v1/courses/{slug}` | Full player state (modules → lessons → completion + notes). |
| `POST /v1/courses/{slug}/progress` | Validates `lesson_id` against `products.specs_json` manifest. |
| `PUT /v1/courses/{slug}/lessons/{lesson_id}/notes` | Notes only; never flips completion. |

### Anti-regressions 56–58

56. `mark_complete`'s ON CONFLICT DO UPDATE MUST NOT touch `notes`. Otherwise marking complete clobbers the user's notes.
57. `update_progress`'s `completed_at = CASE WHEN $2 >= 100 AND completed_at IS NULL THEN now() ELSE completed_at END` is load-bearing. Without the NULL guard, every re-update at 100% refreshes the completion timestamp.
58. `post_progress` MUST validate `lesson_id` against the manifest. Silent accept pollutes the course_progress table.

### Decisions

- **Manifest in `products.specs_json`** — content lives alongside price/title/etc. PR #14's admin CRUD will validate shape on save.
- **`time_spent` accumulates on re-mark** — re-watches are real signal; last-write-wins discards data.
- **Notes stay decoupled from completion** — typical flow is take-notes → click-complete. Separate code paths in the repo.

### Original scope notes (for audit)

**Scope (BACKEND.md §21 row 12):** courses endpoints + progress + notes.

---

## PR #11 — worker binary + indicators + downloads catalog/grants + invoice-PDF fetch job (2026-05-25) — ✅ shipped

**Evidence:** [`backend/docs/launch-evidence/pr-11-worker-indicators-downloads.md`](backend/docs/launch-evidence/pr-11-worker-indicators-downloads.md).

### Surfaces added

| Surface | Notes |
|---|---|
| `bins/worker` | New workspace member. 5-min `tokio::interval` driving reconcile + invoice-PDF sweeps. SIGINT/SIGTERM clean shutdown. |
| `StripeApi::get_invoice` | New trait method; `StripeClient` does `GET /v1/invoices/{id}`. `RecordingStripeApi` returns a deterministic fake. |
| `InvoicesRepo::list_missing_pdf(limit)` | Selects rows where `pdf_r2_key IS NULL`, newest first. |
| `jobs::fetch_invoice_pdfs` | Worker job. Re-hosts Stripe invoice PDFs into R2. Idempotent on retry. |
| `DownloadsCatalogRepo` | `list_for_product`, `list_for_user`, `find_by_id`, `upsert`. |
| `DownloadGrantsRepo::record_access_in_tx` | UPSERT bumps `download_count` atomically inside the caller's tx. |
| `ProductsRepo::find_by_id` | Resolves products regardless of `active`, so the indicators dashboard renders names after a product is retired. |
| `GET /v1/indicators` | List active licenses + product slug/name. |
| `GET /v1/indicators/{slug}/key` | License-key **prefix** only. Plaintext ships once via the mailer (PR #13). |
| `GET /v1/indicators/{slug}/downloads` | Catalog rows the user is entitled to for this product. |
| `GET /v1/downloads` | All catalog rows the user is entitled to. |
| `GET /v1/downloads/{id}/url` | Entitlement check → tx { grant bump + audit } → 300 s presigned R2 GET. |
| Migration 0022 | `CREATE UNIQUE INDEX … ON downloads_catalog (product_id, platform, version)`. Forward-only; original 0016 didn't include it. |

### Anti-regressions 52–55

52. `fetch_invoice_pdfs` MUST use a separate `reqwest::Client` from the StripeClient. Reusing the bearer-auth one would leak our secret to `files.stripe.com`.
53. The catalog upsert depends on migration 0022's UNIQUE index.
54. Grant bump + audit row MUST share one tx; the presign rides outside.
55. `ProductsRepo::find_by_id` deliberately omits `active = TRUE`.

### Decisions

- **Worker as a separate binary** (BACKEND.md §1.7) — keeps the api request path off the critical path of long Stripe/R2 round-trips.
- **`get_invoice` re-fetched per attempt, not stored** — Stripe's invoice_pdf URL is short-TTL and rotates on some plans. Re-fetch means we never re-host from a stale URL.
- **Audit + grant bump in one tx, presign outside** — BACKEND.md §22 rule 8. If presign fails, the audit row stays (correct: we tried), the grant counter doesn't bump (correct: no successful delivery). The reverse (rollback on presign failure) would lose the operational trail.
- **License-key plaintext NOT viewable after issuance.** The endpoint shows the prefix only. The plaintext travels through `IssuedLicense.plaintext` to the mailer (PR #13). Lost-key → admin re-issuance (PR #14).

### Original scope notes (for audit)

**Scope (BACKEND.md §21 row 11):** indicators endpoints + license key issuance + downloads catalog + download grants (presign + audit in one tx). License key issuance was actually delivered in PR #8; this PR adds the user-facing surface that lists/exposes the prefixes + catalog rows + presign + grant tracking.

---

## PR #10 — r2-client + GET /v1/billing/invoices + GET /v1/billing/invoices/{id}/pdf-url (2026-05-25) — ✅ shipped

**Evidence:** [`backend/docs/launch-evidence/pr-10-r2-invoices.md`](backend/docs/launch-evidence/pr-10-r2-invoices.md).

### Surfaces added

| Surface | Notes |
|---|---|
| `r2-client` crate | New workspace member. `ObjectStore` trait + `R2Client` (rusty-s3 + reqwest) + `RecordingObjectStore` in-memory fake. |
| `r2_client::keys` | Stable key conventions: `invoices/{user_id}/{invoice_id}.pdf`, `downloads/{product_id}/{version}/{filename}`, `exports/{user_id}/{job_id}.json.gz`. |
| `InvoicesRepo::attach_pdf_r2_key` | Idempotent UPDATE; **never overwrites a foreign key** (anti-orphan). |
| `InvoicesRepo::find_for_user` | Entitlement-scoped lookup. |
| `GET /v1/billing/invoices` | Returns `[{id, number, status, amount_cents, currency, invoice_date, has_pdf}]`. Hides raw `pdf_r2_key`. |
| `GET /v1/billing/invoices/{id}/pdf-url` | HEAD + 300 s presigned GET. 404 if missing or wrong user. |
| `AppState.r2` field | `Arc<dyn ObjectStore>`. |
| api binary R2 wiring | Real R2 from R2_* env; dev fallback to RecordingObjectStore + WARN; production fail-closed on missing R2_*. |

### Anti-regressions 49–51

49. `attach_pdf_r2_key` MUST keep `WHERE … AND (pdf_r2_key IS NULL OR pdf_r2_key = $2)`. Dropping it makes a retry with a different key orphan the prior R2 object.
50. `InvoicesRepo::find_for_user` is the only entitlement-safe lookup. No `find_by_id` exists, by design — a bare lookup would skip the user_id check.
51. `pdf_url` handler MUST `head_object` before `presigned_get`. Otherwise we hand the user a URL that 403s mid-download when R2 ↔ DB diverge.

### Decisions

- **`rusty-s3` over `aws-sdk-s3`** — ~50 deps lighter, single-purpose. The full AWS SDK is overkill for "PUT a PDF + presign a GET".
- **`has_pdf: bool` in list response, not the raw key** — keys are internal infra; UI only needs the download-link gate.
- **PDF presign TTL = 300 s** — short blast radius for leaked URLs, comfortable for a normal click-through. TTL mirrored to BFF for cache headers.
- **Production fail-closed on missing R2_*** — the recording fallback is dev-only. A silent prod fallback would mean uploads vanish into RAM, lost on next restart.
- **Worker is PR #11.** Until then, every invoice row keeps `pdf_r2_key = NULL`, so the list endpoint reports `has_pdf: false` and the BFF hides the download link. This is intentional, not a bug.

### Original scope notes (for audit)

**Scope (BACKEND.md §21 row 10):** R2/S3 client + invoice PDF presign endpoint. The worker that backfills the PDFs is PR #11.

---

## PR #9 — Customer Portal + invoice.paid + subscription.* + renewal (2026-05-25) — ✅ shipped

**Evidence:** [`backend/docs/launch-evidence/pr-09-portal-and-subscription-handlers.md`](backend/docs/launch-evidence/pr-09-portal-and-subscription-handlers.md).

### Anti-regressions 45–48 (in evidence doc).

### Decisions deferred

- **Plan-swap mapping in `customer.subscription.updated`.** Today we
  refresh status + period + cancel flag; remapping `plan_id` based on
  `items.data[0].price.id` needs `subscription_plans.stripe_price_id`
  populated for real (only placeholders today). PR #14's admin product
  CRUD will land that.

### Original scope notes (for audit):

**Scope (BACKEND.md §21 row 9):** Customer Portal endpoint;
`invoice.paid` (renewals); `customer.subscription.updated`
(plan change / cancel scheduling); `customer.subscription.deleted`
(revocation inline at period end). `charge.refunded` → PR #15.

### Decisions

- **`POST /v1/billing/portal`** returns `{url}` for the BFF to redirect.
  The portal session is created with `return_url = {public_site_url}/dashboard/billing`.
  Idempotency-keyed on `(user_id, "portal_session", session_id_or_timestamp)`
  — short-lived URLs benefit less from idem, but it normalizes the API
  pattern.
- **`invoice.paid`** is the renewal signal. Handler:
  1. Find the subscription by `event.data.object.subscription`.
  2. Refresh `current_period_end` from the event.
  3. Insert an `invoices` row (idempotent on `stripe_invoice_id`).
  4. Audit `subscription.renewed`.
  Receipt email is enqueued by PR #X mailer; today we log.
- **`customer.subscription.updated`** refreshes
  `status` + `cancel_at_period_end` + `current_period_start/end` on the
  mirror. `plan_id` change is detected by mapping
  `data.object.items.data[0].price.id` → our `plan_id` via
  `subscription_plans.stripe_price_id`. If unmapped → log + skip plan
  swap (entitlements still re-derive via the next webhook).
- **`customer.subscription.deleted`** fires at period end. Handler:
  1. Mark subscription `status = 'canceled'`, `canceled_at = now()`.
  2. Revoke every `enrollments` row with `source = 'subscription'`
     for this user (`active = FALSE`). Purchase-source enrollments
     survive.
  3. Audit `subscription.canceled`.
  BACKEND.md §1.7 / §8.4: do NOT schedule a future job for revocation
  — Stripe already fires this event at the exact period_end.

**Scope (BACKEND.md §21 row 8):** subscription endpoints (read-only),
`checkout.session.completed` real handler, entitlements (purchase → license
/ enrollment), reconciliation function (worker wiring is PR #11).

### Scope boundaries

| In | Out |
|---|---|
| Real `checkout.session.completed` dispatcher: orders→paid, invoices, subscriptions mirror, enrollments, licenses, audit_log — all in one tx | `invoice.paid` / `customer.subscription.*` / `charge.refunded` (PR #9 / #15) |
| `LicensesRepo::issue_for_purchase` + license-key generator (`TF-{kind}-{4x4-Crockford}`) | License revocation endpoints (PR #11) |
| `EnrollmentsRepo::create_for_purchase` | Course-progress writes (PR #12) |
| `InvoicesRepo::record` | PDF fetch + R2 upload (PR #10) |
| `SubscriptionsRepo::upsert_from_stripe` | change-plan / pause / cancel (PR #9 — Customer Portal handles them) |
| `AuditRepo::record` | Admin audit-log UI (PR #14) |
| `GET /v1/subscription` (read-only — caller's current sub) | `POST /v1/subscription/*` (PR #9) |
| `reconcile_stripe_events` reusable function (callable from anywhere, exact same `claim → dispatch → mark` flow as the webhook) | The cron that schedules it (PR #11 worker) |

### Decisions

- **Webhook dispatcher branches on `event.kind`.** PR #7's noop becomes
  a real `match event.kind { ... }` with `checkout.session.completed`
  handled and the others logged + skipped. PR #9 adds the next batch.
- **License key format = `TF-{kind}-{4x4-Crockford}` (24 chars).**
  Matches BACKEND.md §8.5. `kind` = 2-letter shortcut (e.g. `RR` for
  Revolution Ranger) derived from the product's `legacy_slug_id`. Hash
  stored as Argon2id (same keyed-pepper instance as passwords);
  plaintext shown ONCE via PR #11's `GET /v1/indicators/{slug}/key`.
- **Enrollments + licenses are created `source = 'purchase'`** with
  `source_ref_id = order_id`. PR #15's refund handler can find them by
  `source_ref_id` and flip `active = false`.
- **Subscription mirror.** When `mode = subscription`, the event payload
  carries `data.object.subscription` (a sub id). The handler upserts a
  row in `subscriptions` from the order's plan + the event's
  current_period_start/end. Stripe is the source of truth; the mirror
  exists so entitlement reads don't hit Stripe (BACKEND.md §1.3).
- **`reconcile_stripe_events` lives in `storage::stripe_events`** as a
  standalone fn that takes a `&dyn Fn(...)` dispatcher — caller-agnostic.
  The webhook handler and the worker call into the same `dispatch` fn.
- **The seeder writes a deterministic placeholder
  `stripe_price_id`** (`price_test_TF_{slug}`) so the `/v1/checkout`
  path compiles + runs end-to-end against the `RecordingStripeApi`.
  Real Stripe prices land later (PR #14 admin product CRUD or one-off
  ops script).
- **For PR #8 evidence we can't drive a real Stripe Checkout** without
  test keys, so the evidence script pre-inserts a pending order +
  fake checkout session id via SQL, then signs + POSTs a synthetic
  `checkout.session.completed` event. This exercises the *exact*
  dispatcher path prod takes; the only thing not exercised is the
  Stripe-side Session creation, which is PR #7's path.

**Scope (BACKEND.md §21 row 7):** `stripe-client` + `checkout` endpoint +
webhook receiver (signature verify, insert-unprocessed → dispatch →
mark-processed, dispatch = noop).

**Verification target:** Stripe CLI signature test; webhook 200; crash-
recovery test.

### Scope boundaries

| In | Out |
|---|---|
| `stripe-client` crate with `StripeApi` trait + real `async-stripe` impl | Real Stripe API calls from CI (no test keys in this evidence run) |
| `OrdersRepo::create_pending` + `OrderItemsRepo` | `paid` / `refunded` transitions (PR #8 / #15) |
| `StripeEventsRepo` with claim → dispatch → mark-processed semantics (BACKEND.md §8.3) | Real event handlers (`checkout.session.completed`, `invoice.paid`, etc.) — they're PR #8+ |
| `POST /v1/checkout` — validates cart against DB, creates pending order, calls Stripe to mint Checkout Session, returns the URL | Subscription-mode checkout (PR #8) and product-vs-plan disambiguation are scaffolded; the actual Stripe price-id wiring lands with PR #8 |
| `POST /v1/webhooks/stripe` — raw-body signature verify, claim-or-skip, dispatch-in-tx, mark-processed-on-success | `reconcile_stripe_events` cron — requires the `worker` binary (PR #11) |
| Required-at-boot config: `STRIPE_SECRET_KEY`, `STRIPE_WEBHOOK_SECRET`, `STRIPE_API_VERSION` | |

### Decisions

- **`StripeApi` trait, two impls.** `StripeClient` (real, async-stripe)
  + `RecordingStripeApi` (test fake — captures every call into an
  in-memory log). Wire either via `Arc<dyn StripeApi>` on `AppState`.
  The api binary always uses the real client; tests + evidence use the
  fake when test keys aren't available.
- **Webhook signature** is verified by feeding the **raw** request body
  to `stripe::Webhook::construct_event` (axum `Bytes` extractor, NOT
  `Json<_>`). This is BACKEND.md §8.3's load-bearing detail.
- **Claim query** is the corrected R2 one from §8.3:
  ```sql
  INSERT INTO stripe_events (event_id, event_type, payload, received_at)
  VALUES ($1, $2, $3, now())
  ON CONFLICT (event_id) DO UPDATE SET event_id = stripe_events.event_id
  RETURNING (xmax = 0) AS freshly_inserted, processed_at
  ```
  The `xmax = 0` trick is the Postgres way to distinguish a fresh
  INSERT from an update that touched an existing row. If freshly
  inserted OR `processed_at IS NULL` → run the dispatcher.
- **Dispatcher is a noop in PR #7.** It logs the event_type, marks
  `processed_at`, and returns `Ok(())`. PR #8 replaces the noop body
  with the real `match event.type_` branches.
- **`POST /v1/checkout` requires auth.** Anonymous checkout isn't a
  thing on the locked Phase-1 UI; the cart page sits behind the same
  cookie surface. Cart contents come in the body; server validates
  each line against current product/plan prices (never trust client
  cents).
- **Idempotency key for Stripe writes:** `tf:{sha256(prefix || user_id
  || action || nonce)[..24]}`, computed in `common::auth::idempotency`
  (moved out of stripe-client so other surfaces — refund, portal —
  use the same helper).
- **Reconciliation sweep is deferred to PR #11** (worker). PR #7's
  evidence demonstrates the row state after a simulated crash so the
  cron-side fix is obvious; the cron itself doesn't exist yet.
- **`STRIPE_SECRET_KEY` and `STRIPE_WEBHOOK_SECRET` required at boot.**
  Same fail-fast pattern as `SERVICE_TOKEN` / `AUTH_*`. Dev workflow
  exports test placeholders; the api will only successfully call Stripe
  when a real test key is set.
- **Reqwest-based Stripe wrapper, not `async-stripe`.** BACKEND.md §3
  starts with `async-stripe = "0.40"`; in practice the `webhook-events`
  feature on 0.40/0.41 fails to compile unless `connect` is also
  enabled — pulling in Stripe Connect APIs we'll never call. The
  webhook signature spec (`t=<ts>,v1=<hex>` header, HMAC-SHA256 over
  `{ts}.{raw_body}` with 5-min tolerance) is 20 lines using primitives
  we already have (`hmac` + `sha2` + `subtle`). REST calls are
  straight `reqwest` against `https://api.stripe.com/v1/...` with
  `application/x-www-form-urlencoded`. Smaller dep tree, no
  type-system whack-a-mole, and (the load-bearing principal call) we
  control the surface so swapping providers later doesn't ripple.
  Per BACKEND.md §23 risk #1: "async-stripe lags Stripe API versions
  historically" — this just front-loads that decision.

**Scope (BACKEND.md §21 row 6):** `public` — products/plans/leads/contact;
rate limiter; SvelteKit BFF rewrites for `/free-guide`, `/contact`.

**Verification target:** evidence + `EXPLAIN ANALYZE` confirming
`leads_created_at_idx` is used; lead-capture limiter fires at the 6th
attempt in a minute; SvelteKit `/free-guide` + `/contact` actions write
through Rust now.

### Scope boundaries

| In | Out |
|---|---|
| `GET /v1/public/products?kind=indicator\|course` | Search / facets (deferred) |
| `GET /v1/public/products/{slug}` | Reviews subsystem (deferred) |
| `GET /v1/public/plans` | Plan upgrade/downgrade preview (PR #8) |
| `POST /v1/public/leads` | Mailing-list provider integration (deferred — log only) |
| `POST /v1/public/contact` | Ticket assignment (PR #14) |
| `ProductsRepo`, `PlansRepo`, `LeadsRepo`, `ContactRepo` | Order/Invoice repos (PR #8) |
| Generalized `rate_limit::Bucket` framework with per-bucket `governor` | Cross-instance (Redis) backend (deferred per §23 risk #2) |
| BFF rewrite of `/free-guide/+page.server.ts` + `/contact/+page.server.ts` | `/admin/leads`, `/admin/messages` BFF rewrite (PR #14) |

### Decisions (planned)

- **Public surface still requires `X-Service-Token`.** Rust has no
  public URL (BACKEND.md §1.1) — the SvelteKit BFF is the only caller,
  even for "anonymous" reads. No user auth required.
- **Rate limiter is generalized into `http::middleware::rate_limit`.**
  Each bucket = one `governor::RateLimiter`. `LoginLimiter` becomes a
  thin re-export. Buckets per BACKEND.md §12:
  - `Public` (60/min IP) — public reads.
  - `LeadCapture` (5/min IP) — `/v1/public/leads`.
  - `Contact` (3/min IP) — `/v1/public/contact`.
  - `Login` (5/min IP) — kept at PR #3 value; raises to 10/min when
    PR #6's `(IP, normalized_email)` key-tuple lands.
- **`ProductsRepo` and `PlansRepo` return the JSONB columns verbatim** —
  the BFF maps to its own UI shape. Cleaner than dressing the row in
  the API surface and then dressing again on the BFF.
- **`LeadsRepo::create` enforces a unique-by-`(email, source)` insert
  with `ON CONFLICT DO NOTHING`** — a returning user re-subscribing to
  the same source is a no-op success, not a 409. (The schema's index
  is non-unique on `email`; the dedupe is a logical decision in the
  repo, not the DB.) Actually for PR #6 we **skip dedupe** and accept
  duplicates — leads are append-only audit; the source field is the
  funnel signal. Re-evaluate when admin/leads UI needs distinct counts.
- **`POST /v1/public/contact`** has a `website` honeypot field (matches
  the existing Phase-1 form). If non-empty → silent 201 (don't tip the
  bot) and don't insert.
- **BFF `rust-client.ts`** centralizes `fetch` to Rust with
  `X-Service-Token` + `X-Request-Id` + cookie pass-through. Drizzle code
  is NOT removed in PR #6 — that's the cutover PR (#17). PR #6 puts the
  Rust call alongside the Drizzle write, gated by `env.USE_RUST_BACKEND
  === 'true'`. Flip the flag, validate, then remove Drizzle in PR #17.
- **No EXPLAIN ANALYZE on cold cache** — the evidence run does a `SELECT
  pg_prewarm('leads')` first so the plan reflects warm-cache behavior
  (the actual production state under load).

**Scope (BACKEND.md §21 row 5):** TOTP enable/confirm/disable + backup
codes; `skew=1` pinned in code.

**Verification target (BACKEND.md §16.3, the 2FA item):**
- enable → scan QR → confirm code → 10 backup codes shown once →
  `users.totp_enabled_at IS NOT NULL`.
- logout → wrong TOTP → 401 → right TOTP → 200.
- a code from the previous 30 s step is still accepted (skew = 1).
- a code from two steps ago is rejected.
- a backup code consumes a slot.

### Scope boundaries

| In | Out |
|---|---|
| `POST /v1/me/2fa/enable` → fresh secret, encrypted at rest, returns otpauth URI + QR data URL | Webauthn / U2F (deferred) |
| `POST /v1/me/2fa/confirm` → verify first code, set `totp_enabled_at`, issue 10 backup codes | Trusted-device / "remember this browser" (deferred) |
| `POST /v1/me/2fa/disable` → requires current password + current TOTP | Recovery via admin override (PR #14) |
| `POST /v1/me/2fa/backup-codes/regenerate` → replaces all codes; old ones invalidated | Per-code labels / scratch-list UI metadata |
| `POST /v1/auth/login` two-step branch: returns `{step:"totp", tx_id}` when 2FA is on | Time-based throttle on TOTP (handled by existing login limiter on stage 1 — the limiter sees one bucket-consumption per `/login` hit, before `/login/totp`) |
| `POST /v1/auth/login/totp` → verifies code (or backup code), mints session | |

### Decisions

- **TOTP params pinned in code (BACKEND.md §1.2 / §7.3):** SHA-1, 6 digits,
  30-second period, `skew = 1` (current step ± 1, ≈90 s validity).
  Every literal is passed to the `totp-rs::TOTP::new` constructor — never
  relying on a crate default.
- **Secret-at-rest:** 20-byte secret, encrypted with `XChaCha20-Poly1305`
  using `AUTH_TOTP_KEY` (32 bytes from env). Ciphertext layout in
  `users.totp_secret_encrypted` is `[nonce:24][ciphertext+tag:36]` →
  60 bytes total. Nonce is freshly random per encryption.
- **Backup codes:** 10 codes of 10 chars Crockford base32 (no I, L, O, 0
  ambiguity), Argon2id-hashed via the same `argon2_with_pepper()` builder
  as passwords. Stored in `users.twofa_backup_codes_hash` (`BYTEA[]`)
  as PHC strings encoded UTF-8. **Consumed slots are replaced with
  all-zero `BYTEA`** so the array length stays at 10 (BACKEND.md §7.3 —
  lets the UI show "X/10 remaining").
- **Hashing 10 backup codes costs ~3 s wall-clock at issuance.** That's
  the price of Argon2id for low-entropy secrets. Issuance is a one-off
  per user; the latency is acceptable for an explicit user-initiated
  operation. Each hash goes through `hash_semaphore` to stay inside the
  OOM cap.
- **Pending-TOTP token (stage 1 → stage 2):** signed token of the form
  `v1.<b64url(user_id)>.<b64url(expires_unix_secs)>.<b64url(hmac_sha256(cookie_key, ...))>`.
  Five-minute TTL. **No DB row** — the token IS the state, HMAC-verified
  exactly like the session cookie but unrelated to it. Reuses
  `AUTH_COOKIE_KEY` (a separate `AUTH_TOTP_TX_KEY` would be cleaner; the
  reuse is acceptable because the domain separators (`tfx_session` vs.
  `tfx_pending_totp`) sit inside the HMAC payload).
- **Login flow:** stage 1 (`POST /v1/auth/login`) verifies password under
  the lockout limiter. If `users.totp_enabled_at IS NOT NULL`, response
  is `{"step":"totp","tx_id":"v1...."}` with **no Set-Cookie**. Stage 2
  (`POST /v1/auth/login/totp`) verifies the code (TOTP or backup), THEN
  sets the cookie. Both stages contribute to the per-IP limiter bucket.
- **Backup-code path:** `/login/totp` accepts either a 6-digit TOTP code
  OR a 10-char backup code in the same `code` field, distinguished by
  length. On match, the slot is zeroed in the array.
- **Disable rule:** requires both `current_password` AND `current_totp`.
  Clearing the secret without checking the current code would let a
  shoulder-surfed password disable 2FA, defeating the point.
- **`AUTH_TOTP_KEY` becomes required at api boot.** Fail-fast at
  `Config::from_env`, matching `SERVICE_TOKEN` / `AUTH_COOKIE_KEY` /
  `AUTH_PASSWORD_PEPPER` from PR #3.

### Open items / explicit risks

- **Key rotation for `AUTH_TOTP_KEY` is not handled.** If the key
  changes, every encrypted secret in the DB becomes garbage. Phase 3:
  add a `key_version` byte at the head of the ciphertext + a config
  for the old key, decrypt-old + re-encrypt-new on next login.
- **QR generation uses `qrcode` crate (Rust-side, deterministic).**
  Output is a base64 PNG data URL — the BFF / browser can drop it
  straight into `<img src="...">`.

**Scope (BACKEND.md §21 row 4):** `me` — profile, change-email,
change-password, sessions list/revoke.

**Verification target (BACKEND.md §16.3):** evidence checklist; manual
sessions revoke.

### Scope boundaries

| In | Out |
|---|---|
| `GET /v1/me` → current user payload | Active-sub + entitlement summary (PR #8+) |
| `PATCH /v1/me` → name, headline, timezone, language | Avatar upload (deferred) |
| `POST /v1/me/change-email` → stages a row in `email_verifications`; logs the token | Real email send via Resend (PR #X, after mailer crate exists) |
| `POST /v1/me/change-password` → requires current password | Forced password rotation (deferred) |
| `GET /v1/me/sessions` → list active sessions for the caller | Geo-IP enrichment (deferred) |
| `DELETE /v1/me/sessions/{id}` | Push-style logout notification (deferred) |
| `POST /v1/me/sessions/revoke-others` | |
| `UsersRepo::update_profile`, `update_password`, `email_verifications` repo | TOTP-protected endpoints (PR #5) |

### Decisions (planned)

- **Email-change flow is two-step, but PR #4 ships step 1 only.** The
  endpoint creates an `email_verifications` row with `kind='email_change'`
  and `new_email` populated. The verification-link send is logged via
  `tracing` (the token appears in the JSON log line, prefixed `EMAIL_STUB`
  so it's easy to filter). When the mailer crate lands, swap the log for
  `mailer::enqueue`. **Critical:** the `email_change` row is consumed in
  PR #X (verify-email endpoint) — until then, change-email is a no-op
  from the user's perspective in prod, but staged correctly in the DB.
- **Change-password requires the current password** AND **revokes every
  other session** (BACKEND.md §7.5 spirit — rotating a credential should
  expire other agents holding it). The current session stays alive so the
  caller isn't logged out by their own action.
- **No /v1/me/avatar** endpoint until R2 lands (PR #10). The avatar_url
  column on users exists but stays NULL.
- **`/v1/me/sessions` lists only the caller's own rows.** No admin
  override here — that ships in PR #14's `/v1/admin/customers/{id}`.
- **DELETE /v1/me/sessions/{id} on `id == current_session`** is allowed:
  it's an alternate path to logout from a different device.
- **All endpoints require auth via `require_auth` middleware** introduced
  in PR #3; no public surface added in PR #4.

**Scope (BACKEND.md §21 row 3):** `auth` — signup/login/logout, sessions,
Argon2id keyed pepper (hash + verify); `service_token` middleware.

**Verification target (BACKEND.md §16.3):**
- Signup → 201, DB row exists, `Set-Cookie` has `HttpOnly Secure SameSite=Lax`.
- Password verify round-trip across process restart (pepper survives).
- Login lockout: 11 wrong-password attempts from one IP → 429.
- `service_token` middleware rejects requests without `X-Service-Token`.

### Scope boundaries (what is and isn't in PR #3)

| In | Out |
|---|---|
| `UsersRepo::create/find_by_email/find_by_id` | `update_password` (PR #4) |
| `SessionsRepo::create/load/revoke/revoke_all_except` | TOTP-protected login (PR #5) |
| Argon2id `hash_password` + `verify_password` with keyed pepper + `spawn_blocking` + `Semaphore(8)` | Email verification flow (PR #4) |
| Session cookie: `tfx_session = v1.<id>.<token>.<hmac>` | Password reset (PR #4) |
| `AuthSession` extractor (cookie → user_id + session_id) | `/v1/me/*` endpoints (PR #4) |
| `service_token::verify` middleware (X-Service-Token + IP allowlist) | Cross-instance rate limiter (PR #6) |
| `POST /v1/auth/signup`, `POST /v1/auth/login`, `POST /v1/auth/logout` | Forgot-password (PR #4) |
| Per-IP login attempt limiter (5/min, returns 429) | General rate-limit framework (PR #6) |

### Decisions (so the future me doesn't second-guess)

- **`SERVICE_TOKEN` is required at api-binary boot.** No "open in dev, closed
  in prod" split — that's the pattern that ships dev secrets to staging by
  accident. Dev sets `SERVICE_TOKEN=dev-token` in env / .env; the smoke
  target sends the matching header.
- **Argon2id pepper as `secret` (BACKEND.md §1.2 / §7.2):** uses
  `Argon2::new_with_secret`. Never concatenated onto the password bytes —
  doing so creates a hash-vs-verify asymmetry that's easy to get wrong
  silently.
- **`Semaphore(8)` caps concurrent hashes.** Lives in `AppState`, threaded
  through every signup/login/verify call. Without it, 100 simultaneous
  logins would each grab a `spawn_blocking` slot at m=64 MiB and OOM a
  small Railway box.
- **Session token = 32 random bytes from `OsRng`.** Cookie value is
  `v1.<b64url(session_id)>.<b64url(token)>.<b64url(hmac_sha256(key, "v1." + id + "." + token))>`.
  Server: parse → constant-time HMAC verify → sha256(token) → lookup by
  `token_hash`. We never store the plaintext token; an attacker dumping
  the DB cannot resurrect a session.
- **`last_seen_at` write-coalescing deferred.** BACKEND.md §7.1 calls for
  60 s coalescing via `moka`. PR #3 just writes on every load; PR #4 or a
  follow-up adds the cache once we have a workload to measure against.
- **Login attempt rate limit:** in-process per-IP bucket (5 attempts /
  minute, burst 5). Backed by `governor` + `DashMap`. Lives behind a tiny
  trait so PR #6's general rate-limit framework can swap in without
  touching auth handlers. **Single-instance only;** documented openly.
- **No CSRF token in PR #3.** The session cookie is `SameSite=Lax`, the
  service_token gate is server-to-server, and the BFF (SvelteKit) handles
  browser-facing CSRF natively for form actions. Re-evaluate if/when a
  public Rust surface is ever introduced.
