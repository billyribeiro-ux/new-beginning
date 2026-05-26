# PR #17 — Cutover: Drizzle/SQLite removal + production wiring + flag flip

## Scope (BACKEND.md §21 PR #17)

This is the cutover PR. Three deliverables:

1. **Flag flip (default-on).** `useRustBackend()` in
   `src/lib/server/rust/client.ts` flips from opt-in (`=== 'true'`) to
   opt-out (`!== 'false'`). The Rust backend is the default; setting
   `USE_RUST_BACKEND=false` falls back to the legacy Drizzle paths
   (kept for a single rollback window).
2. **Drizzle/SQLite removal plan.** This document enumerates every file
   to delete + the deletion order. The deletions land as a follow-up
   PR once the cutover is observed clean in production (typically 7
   days).
3. **Production env wiring.** Documented below — every env var the api
   + worker binaries require, and the Vercel BFF env vars.

## Why the rollback window exists

Hard-deleting the Drizzle paths the same day as the flag flip means a
production regression has no escape hatch. Keeping the code alive
behind `USE_RUST_BACKEND=false` for ~7 days lets us revert without a
deploy if a webhook handler edge case (or a Stripe API quirk) surfaces.
After the window, the files below are deleted in a separate PR.

## Production env wiring

### `api` binary (Railway service)

Required:
- `BIND_ADDR=0.0.0.0:8080`
- `METRICS_BIND=0.0.0.0:9090`
- `ENV=production`
- `DATABASE_URL=postgres://…`           — Railway-provisioned.
- `DATABASE_MAX_CONN=20`                — BACKEND.md §13 default.
- `DATABASE_MIN_CONN=2`
- `SERVICE_TOKEN=<32+ random bytes>`    — shared with Vercel BFF.
- `SERVICE_TOKEN_IP_ALLOWLIST=<vercel-egress-CIDR>` — optional but recommended.
- `AUTH_COOKIE_KEY=<32 bytes b64>`      — HMAC for session cookies.
- `AUTH_PASSWORD_PEPPER=<32 bytes b64>` — Argon2id `secret` param.
- `AUTH_TOTP_KEY=<32 bytes hex>`        — XChaCha20-Poly1305 key.
- `AUTH_SESSION_TTL_DAYS=30`
- `AUTH_COOKIE_DOMAIN=<your-domain>`    — host-scoped on preview deploys.
- `STRIPE_SECRET_KEY=sk_live_…`
- `STRIPE_WEBHOOK_SECRET=whsec_…`
- `STRIPE_API_VERSION=2024-12-18.acacia`
- `PUBLIC_SITE_URL=https://…`           — Checkout success/cancel base.
- `R2_ENDPOINT=https://<account>.r2.cloudflarestorage.com`
- `R2_BUCKET=tradeflex-prod`
- `R2_ACCESS_KEY_ID=…`
- `R2_SECRET_ACCESS_KEY=…`
- `OTEL_EXPORTER_OTLP_ENDPOINT=https://…`  — Tempo/Honeycomb/etc.
- `OTEL_SERVICE_NAME=tradeflex-api`
- `RUST_LOG=info,sqlx=warn`

**Fail-closed in production**: missing `STRIPE_*` / `AUTH_*` / `R2_*` env
vars abort boot with a clear message. There is no "dev secret if unset"
fallback (anti-regression #2 in PR #3 notes).

### `worker` binary (Railway service)

Same env vars as api (it shares `AppState` construction). No
`BIND_ADDR`/`METRICS_BIND` needed — worker doesn't serve traffic.

### `seeder` binary (run-once, then delete)

Just `DATABASE_URL`. Local-dev convenience; production runs use the
Railway `Run command` UI.

### Vercel BFF (`SvelteKit`)

- `USE_RUST_BACKEND=true`              — optional once PR #17 flips the default; explicit for safety.
- `RUST_API_BASE_URL=https://<api>.railway.app`
- `SERVICE_TOKEN=<same as api>`
- `DATABASE_URL=file:./drizzle/legacy.db` — kept until rollback window expires; then deleted.
- The usual Vercel build envs.

## Drizzle deletion plan (follow-up PR, after rollback window)

Files / paths to delete in order:

1. `src/lib/server/db/seed.ts`, `src/lib/server/db/schema.ts`,
   `src/lib/server/db/client.ts`, `src/lib/server/db/` (entire dir).
2. All `+page.server.ts` Drizzle branches:
   - `src/routes/contact/+page.server.ts` — remove the `if
     (!useRustBackend())` fallback.
   - `src/routes/free-guide/+page.server.ts` — same.
   - `src/routes/admin/+page.server.ts`, `admin/leads/+page.server.ts`,
     `admin/messages/+page.server.ts` — replace Drizzle queries with
     `callRust('/v1/admin/...')`.
3. `drizzle/local.db` and `drizzle/` migration dir.
4. `drizzle.config.ts`.
5. Remove `drizzle-orm`, `drizzle-kit`, `@libsql/client` from
   `package.json`. Run `npm install` to refresh the lockfile.
6. Remove the `DATABASE_URL` env entry pointing at `file:./` — the
   only DATABASE_URL after deletion belongs to the Rust api binary.

Each file deletion is independent + reviewable. Recommend a single PR
per `+page.server.ts` for clean revert.

## What stays

- `src/lib/server/rust/client.ts` — the BFF call helper.
- All `+page.svelte` files (no DB knowledge in the components).
- Form actions that already call `callRust` — they keep working.

## Anti-regressions 72–73

72. The flag flip is opt-out (`!== 'false'`) not implicit (`!=
    undefined`). A missing env now defaults Rust ON. Reverting requires
    explicit `USE_RUST_BACKEND=false`.
73. The Drizzle deletions are deferred to a separate PR. Mixing the
    flag flip with file removal removes the rollback escape hatch.

## Gates

This PR is a single-line change to `client.ts` + this doc. No code
behavior change beyond the default. All workspace gates (fmt / clippy /
test / deny) remain green from PR #16.

## Cutover checklist (operator runbook)

Before flipping the env on Vercel:

- [ ] Railway api binary green; `/healthz` 200, `/readyz` 200.
- [ ] Railway worker binary green (logs show "reconcile sweep: no
      candidates" tick).
- [ ] Stripe webhook endpoint pointed at
      `https://<vercel>/api/webhooks/stripe` (NOT at Railway directly).
- [ ] BFF env vars match the api's `SERVICE_TOKEN`.
- [ ] Smoke test: `POST /v1/auth/signup` → 200 → cookie present →
      `GET /v1/me` → 200.
- [ ] Smoke test: `POST /v1/public/leads` → 200 → row in `leads` table.
- [ ] Smoke test: Stripe CLI `stripe trigger checkout.session.completed` →
      audit row in `audit_log`.

After 7 days of clean traffic + no rollback needed, open the Drizzle
deletion follow-up PR.
