# TradeFlex Backend — Rust + Axum + Tokio + sqlx + Postgres

**Status:** Implementation plan (Phase 2 — backend cutover). **Revision 2** —
incorporates the flaw-fix pass (15 items, see §26 changelog).
**Audience:** the engineer (or LLM agent) executing the backend buildout.
**Authority:** this document is the source of truth. If `STACK.md` or `CHANGELOG.md`
disagree, this document wins for backend concerns; update them after the fact.

---

## 0. Context — why this exists

Phase 1 shipped a SvelteKit + SQLite/Drizzle scaffold. It is functionally a
marketing site with a stubbed dashboard, mock auth, and two routes (`/free-guide`,
`/contact`) that actually persist to a local SQLite file. The frontend UI is
fully built out for `/dashboard/**` and `/admin/**` against mock data — the
visual contract is locked in.

Phase 2 replaces every server-side surface with a production Rust backend:

- Real auth (Argon2id, sessions, TOTP 2FA, backup codes).
- Real catalog + ordering (Stripe Checkout + Subscriptions + Customer Portal).
- Real entitlements (course enrollments, indicator licenses, R2-backed downloads).
- Real ops surface (admin KPIs, leads, messages, products, customers).
- Real observability (tracing, metrics, healthz/readyz).
- Hard-evidence verification per feature (CLAUDE.md habit #2).

**Out of scope for Phase 2:** mobile apps, public-facing third-party APIs,
multi-tenant isolation, analytics warehouse. All deferred.

---

## 1. Architectural decisions — locked in

These were made with Principal-Engineer-Level-7+ judgment. Do **not** revisit
without an RFC and an explicit reason. The rationale lives in this document so
future readers can grade the trade.

### 1.1 Bridge model — **BFF (SvelteKit-in-front, Rust-behind)**

- Browser ↔ SvelteKit (Vercel or Railway). SvelteKit handles cookies, CSRF,
  SSR, edge cache, form actions, page-level rate limit.
- SvelteKit `+page.server.ts` and `+server.ts` ↔ Rust API over **private
  Railway DNS**. No public Rust URL. No browser→Rust direct.
- **Service-to-service auth — one canonical name.** The SvelteKit BFF
  authenticates to Rust with a single header: **`X-Service-Token: <SERVICE_TOKEN>`**.
  Not `Authorization: Service …`, not a Rust constant spelled `X_SERVICE_TOKEN`
  used inconsistently — the wire header is the lowercase-hyphenated
  `x-service-token`, the Rust `HeaderName` constant is
  `HeaderName::from_static("x-service-token")`, and every reference in this
  document and the codebase uses that spelling. Verified by the
  `service_token::verify` middleware (§6) plus a CIDR IP allowlist
  (`SERVICE_TOKEN_IP_ALLOWLIST`).
- **No CORS layer.** Because there is no public Rust URL and no browser ever
  issues a cross-origin request directly to Rust, a `CorsLayer` is dead code
  and a future foot-gun (someone would assume it is load-bearing). It is
  deliberately omitted from §6. If a public Rust surface is ever introduced,
  CORS gets designed then, with intent — not inherited by accident.
- Why this shape: preserves the form-action progressive-enhancement story,
  keeps Stripe secrets and DB creds off the browser-facing service entirely,
  gives us one place (SvelteKit) to set `Set-Cookie` / `Cache-Control` /
  `Vary` headers correctly. The 5–10 ms extra hop is irrelevant inside a
  single Railway region.

### 1.2 Auth — **opaque sessions + Argon2id + real TOTP**

- HMAC-signed opaque session cookie (`tfx_session`), `HttpOnly Secure
  SameSite=Lax`. Server-side session table; instant revocation.
- Argon2id (m=64 MiB, t=3, p=1). **The pepper is supplied as Argon2's keyed
  `secret` parameter via `Argon2::new_with_secret`, not concatenated onto the
  password bytes.** Keyed peppering binds the pepper into the MAC; concatenation
  is strictly weaker and creates a hash/verify asymmetry that is easy to get
  wrong. See §7.2 for the full hash *and* verify paths. Hashing runs inside
  `tokio::task::spawn_blocking`, gated by a `Semaphore(8)` so concurrent logins
  can't OOM a small Railway box.
- TOTP via `totp-rs` (SHA-1, 6 digits, 30 s period). **Skew is pinned
  explicitly in code: `skew = 1`, meaning one step on each side — the current
  step plus the immediately previous and next steps are accepted (≈90 s total
  validity). This is set as a literal in the `TOTP` constructor, never left to
  a crate default.** Secret encrypted at rest with `XChaCha20-Poly1305` using
  `AUTH_TOTP_KEY`. Ten one-shot backup codes, each Argon2id-hashed.
- **No JWT.** The dashboard already has a "list sessions / sign out
  everywhere" UI — JWTs without server lookup can't do that, defeating their
  point.

### 1.3 Stripe — **full real flow on test keys**

- Hosted Stripe Checkout for one-time products + subscription start.
- Stripe Subscriptions for plans (monthly / quarterly / annual).
- Embedded Stripe Customer Portal for billing self-serve.
- Signature-verified webhooks with a `stripe_events` table that serves **two
  distinct jobs that this revision keeps separate**: (a) idempotency, keyed on
  `event_id` PK, and (b) a `processed_at` lifecycle so a webhook that is
  *received* but not yet *processed* is visible and re-drivable. The raw
  payload is stored in its own nullable column purely for replay/debugging and
  is **not** load-bearing for dedupe — see §8.3.
- Idempotency keys on every Stripe **write** (`tf:{base32(sha256(prefix||user||action||nonce))[..24]}`).
- Stripe state is the source of truth; our DB mirrors it. We pay one
  webhook-lag of staleness in exchange for sub-100 ms entitlement reads.
- Stripe CLI drives E2E tests (`stripe trigger checkout.session.completed`).

### 1.4 Object storage — **Cloudflare R2, presigned-after-entitlement**

- `aws-sdk-s3` pointed at the R2 S3-compatible endpoint
  (`region("auto")`, `force_path_style(true)`).
- Presigned GETs minted **only after** server-side entitlement check.
- TTL: 300 s for indicator binaries + invoice PDFs; 24 h for data exports
  (one knowing exception, documented at the call site).
- Behind a `trait ObjectStore` so tests can swap in LocalStack.

### 1.5 Money — **i64 cents end-to-end, no exceptions, no silent loss**

- Postgres `BIGINT`. Rust `common::money::Money(i64)` newtype.
- **No** `impl Mul<i64> for Money`, **no** `impl Mul<Money, Money>`. The only
  multiplication APIs are `mul_bps(i32)` and `mul_pct(i32)`. **Both return
  `Result<Money, MoneyError>`** — they widen to `i128`, divide, and then
  `i64::try_from` the result. They do **not** saturate. Saturation on a money
  value is a silently wrong answer, which is the exact failure mode the newtype
  exists to prevent. `price * qty` still fails to compile; an overflowing
  `mul_bps` returns `Err`, never a clamped lie.
- Serde guards against silent JS precision loss **symmetrically**: both
  `Serialize` and `Deserialize` refuse any value outside
  `[-(2^53-1), 2^53-1]`. A malformed BFF payload is rejected at the
  deserialize boundary, not deferred to explode on the next serialize.

### 1.6 Database — **Postgres 16, text+CHECK enums, uuid v7 PKs**

- `sqlx` with compile-time query checking. `.sqlx/` cache is committed.
- Forward-only migrations under `backend/migrations/NNNN_description.sql`.
  This revision defines exactly **0001–0021**; there is no `0022`. Any
  document or table that referenced "0001–0022" was wrong and is corrected.
- Enums: `TEXT NOT NULL CHECK (col IN (...))`, never native `CREATE TYPE`.
  Adding a value is a one-line ALTER, not a coordinated deploy.
- PKs: `Uuid::now_v7()` (time-sortable), **generated in Rust**. No table has a
  database-side UUID default; the migrations deliberately omit
  `DEFAULT gen_random_uuid()` everywhere. Legacy slug IDs preserved in
  `legacy_slug_id` columns where external systems reference them.
- `CITEXT` for emails — Drizzle's plain `TEXT UNIQUE` allowed `foo@x.com`
  and `FOO@x.com` to both register.
- **Why `pgcrypto` is still installed (§4, `0001`):** *not* for UUID
  generation (PKs come from Rust). It is required for `digest()` / `gen_salt()`
  used by ad-hoc operational queries and any future server-side hashing in
  migrations. The extension is kept with that single documented purpose; if a
  later audit finds nothing uses it, drop it in a forward migration.

### 1.7 Background work — **separate `worker` binary, Postgres-backed queue**

- `background_jobs` table polled with `SELECT … FOR UPDATE SKIP LOCKED`.
- Retry backoff `[60s, 5m, 30m, 2h, 12h]`, max 5 attempts → dead-letter.
- Cron via an in-worker scheduler that enqueues with
  `INSERT … ON CONFLICT (kind, schedule_key) DO NOTHING`.
- Single worker for v1; **every job kind must be safe to run twice** (the
  contract `SKIP LOCKED` gives us is at-least-once, not exactly-once).
- Postgres job queue ceiling is ~50 jobs/sec. Document the exit path
  (Redis/SQS) but don't pre-build it.
- **Delayed jobs use `run_at` only for work Stripe will not re-signal.** We do
  **not** schedule a future `revoke_subscription_entitlements` job to fire at
  `current_period_end` — Stripe already emits `customer.subscription.deleted`
  at exactly that moment, and self-scheduling duplicates Stripe's clock and
  invites drift. Revocation happens directly on the `deleted` event (§8.4).
  `run_at` is reserved for things like `expire_pending_order` where no external
  signal exists.

### 1.8 Email — **Resend + askama, always async via the queue**

- `resend-rs` wrapped in a `trait Mailer` so tests use a recording mock.
- Templates: MJML source → `templates/{name}.html` + `templates/{name}.txt`
  compiled by `build.rs`. Both HTML and plaintext emitted for every send.
- Every email is enqueued, never sent inline. Keeps request latency stable
  when Resend's tail is slow.

### 1.9 Observability — **tracing JSON + Prometheus + OTLP traces + bounded labels**

- `tracing` JSON to stdout. Spans keyed by **matched route**, never raw URI
  (cardinality discipline).
- Prometheus on a separate listener (`:9090/metrics`).
- **OpenTelemetry is wired in Phase 2, not gestured at.** A `tracing-opentelemetry`
  layer exports spans over OTLP to the collector named by `OTEL_EXPORTER_OTLP_ENDPOINT`.
  If that env var is unset, the OTLP layer is simply not installed (local dev,
  no collector) — the JSON stdout layer always runs regardless. The earlier
  draft carried an `OTEL_SERVICE_NAME` env var with no exporter behind it; this
  revision either makes it real or it isn't mentioned. It is now real.
- `/healthz` = "process alive". `/readyz` = "DB reachable in <100 ms AND
  Stripe last-ping was <30 s ago and OK".
- `x-request-id` propagated via middleware + `tokio::task_local` so
  `AppError::into_response` can attach it to error JSON.

### 1.10 Hosting — **Railway (Rust + Postgres + worker) + Vercel or Railway (SvelteKit)**

- One Railway environment, three services: `api`, `worker`, `postgres`.
  Optional fourth: `svelte` if we move the BFF off Vercel for private DNS.
- R2 is Cloudflare-side (account-bound).
- Resend is SaaS (DNS + DKIM).

---

## 2. Repository layout

```
new-beginning/                        # existing repo root
├── src/                              # existing SvelteKit app (BFF)
├── backend/                          # NEW — Rust workspace lives here
│   ├── Cargo.toml                    # [workspace] resolver = "2"
│   ├── Cargo.lock
│   ├── rust-toolchain.toml           # channel = "stable"
│   ├── .sqlx/                        # offline query cache (committed)
│   ├── migrations/
│   │   ├── 0001_extensions.sql
│   │   ├── 0002_users.sql
│   │   └── …                         # 0001–0021, see §4
│   ├── seeds/
│   │   └── catalog.json              # dumped from src/lib/data/* by pnpm run dump-catalog
│   ├── crates/
│   │   ├── common/                   # Money, AppError, ids, config
│   │   ├── domain/                   # business types + ports (traits)
│   │   ├── storage/                  # sqlx repos (concrete, no traits)
│   │   ├── http/                     # Axum routes + middleware + extractors
│   │   ├── stripe-client/            # async-stripe wrapper
│   │   ├── r2-client/                # aws-sdk-s3 wrapper
│   │   ├── mailer/                   # resend-rs + askama
│   │   ├── jobs/                     # job kinds + handlers
│   │   ├── observability/            # tracing + OTLP + metrics setup
│   │   └── test-support/             # fixtures, RecordingMailer, db harness
│   ├── bins/
│   │   ├── api/                      # binary: HTTP server
│   │   ├── worker/                   # binary: background runner
│   │   └── seeder/                   # binary: one-shot catalog upsert
│   ├── docker-compose.dev.yml        # postgres + localstack
│   ├── docker-compose.test.yml       # same, isolated for CI
│   ├── justfile                      # local dev commands
│   └── deny.toml                     # cargo-deny config
└── .github/workflows/backend.yml     # CI pipeline (§17)
```

**Crate dependency graph** (acyclic, fan-in toward `http`):

```
common ◄── domain ◄── storage ◄── http
                  ◄── stripe-client ──►
                  ◄── r2-client     ──► http
                  ◄── mailer        ──►
                  ◄── observability ──► http, worker
                  ◄── jobs          ──► worker
                  ◄── jobs          ──► http (enqueue only)
```

**Rules**:

- `http` is the **only** crate that knows about Axum.
- `storage` is the **only** crate that uses `sqlx::query!` macros.
- `domain` has zero infra deps (no tokio, no sqlx, no axum, no reqwest).

---

## 3. Cargo dependencies (pinned to current stable, May 2026)

Declared in `[workspace.dependencies]` at the workspace root; member crates
reference with `.workspace = true`.

> **Version-verification rule.** Every version below is a *starting point*.
> Before PR #1 is opened, run `cargo add --dry-run` (or check crates.io) for
> each crate and pin to the latest compatible release. Two pairings in
> particular have moved historically and **must** be re-checked: (a) `askama`
> merged its Axum integration into the main crate — recent `askama` no longer
> needs a separate `askama_axum`; if the current release behaves that way, drop
> `askama_axum` and use the `askama`-native Axum response support. (b)
> `async-stripe` tracks Stripe API versions loosely; confirm the feature names
> still exist.

```toml
# Async runtime + HTTP
tokio = { version = "1.42", features = ["full"] }
axum = { version = "0.8", features = ["macros", "json", "tokio"] }
tower = { version = "0.5", features = ["util", "timeout"] }
tower-http = { version = "0.6", features = [
    "trace", "compression-gzip", "request-id",
    "timeout", "limit", "catch-panic"
] }                                                  # NOTE: no "cors" feature — see §1.1
hyper = "1.5"

# DB
sqlx = { version = "0.8", features = [
    "postgres", "runtime-tokio-rustls", "macros",
    "uuid", "time", "json", "migrate", "ipnetwork"
] }

# Serde
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_with = "3"

# Crypto / auth
argon2 = { version = "0.5", features = ["std"] }
secrecy = { version = "0.10", features = ["serde"] }
totp-rs = { version = "5.6", features = ["gen_secret", "otpauth"] }
hmac = "0.12"
sha2 = "0.10"
subtle = "2.6"
rand = "0.8"
base64 = "0.22"
chacha20poly1305 = "0.10"

# Time / IDs
time = { version = "0.3", features = ["serde", "macros"] }
uuid = { version = "1.11", features = ["v4", "v7", "serde"] }

# Validation
validator = { version = "0.19", features = ["derive"] }

# Observability
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
tracing-opentelemetry = "0.28"
opentelemetry = "0.27"
opentelemetry_sdk = { version = "0.27", features = ["rt-tokio"] }
opentelemetry-otlp = { version = "0.27", features = ["grpc-tonic"] }
metrics = "0.24"
metrics-exporter-prometheus = "0.16"

# Errors
thiserror = "2"
anyhow = "1"

# HTTP clients + integrations
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls", "gzip"] }
async-stripe = { version = "0.40", default-features = false, features = [
    "runtime-tokio-hyper-rustls",
    "webhook-events", "checkout", "billing", "customer"
] }
aws-sdk-s3 = { version = "1.60", features = ["rustls"] }
aws-config = "1.5"
resend-rs = "0.10"

# Templates — see §3 version-verification rule re: askama_axum
askama = { version = "0.13", features = ["with-axum"] }

# Rate limit + cache
governor = "0.7"
moka = { version = "0.12", features = ["future"] }

# Utility
async-trait = "0.1"
futures = "0.3"
bytes = "1"
url = "2"
once_cell = "1"
figment = { version = "0.10", features = ["env", "toml"] }
tokio-util = { version = "0.7", features = ["rt"] }   # CancellationToken
```

Per-crate dev dependencies live in each member crate's own `[dev-dependencies]`
table (there is no `[workspace.dev-dependencies]` table — that is not a real
Cargo manifest section). The common set, declared where used:

```toml
[dev-dependencies]
http-body-util = "0.1"
mockall = "0.13"
fake = "3"
insta = "1"
testcontainers = "0.23"
testcontainers-modules = { version = "0.11", features = ["postgres"] }
proptest = "1"
```

Load testing uses the **`oha` binary** (`cargo install oha`, or the Nix/brew
package) invoked from the `justfile` and CI — `oha` is a CLI tool, not a
library, and is never a `[dependencies]` entry.

`jsonwebtoken` is intentionally absent — sessions are opaque.

---

## 4. Postgres schema — full migrations (0001–0021)

All cents columns `BIGINT NOT NULL CHECK (col >= 0)`. All timestamps
`TIMESTAMPTZ NOT NULL DEFAULT now()`. All emails `CITEXT`. All PKs `UUID`
generated server-side with `Uuid::now_v7()` — **no DB-side UUID default
anywhere**.

### 0001_extensions.sql

```sql
-- citext: case-insensitive email uniqueness.
-- pgcrypto: digest()/gen_salt() for operational queries and future in-migration
--           hashing. NOT used for UUID generation — PKs come from Rust now_v7().
CREATE EXTENSION IF NOT EXISTS pgcrypto;
CREATE EXTENSION IF NOT EXISTS citext;
```

### 0002_users.sql

```sql
CREATE TABLE users (
    id                       UUID PRIMARY KEY,
    email                    CITEXT NOT NULL UNIQUE,
    name                     TEXT NOT NULL,
    headline                 TEXT,
    timezone                 TEXT NOT NULL DEFAULT 'UTC',
    language                 TEXT NOT NULL DEFAULT 'en',
    password_hash            TEXT,
    role                     TEXT NOT NULL DEFAULT 'member'
                                CHECK (role IN ('member','admin')),
    email_verified_at        TIMESTAMPTZ,
    stripe_customer_id       TEXT UNIQUE,
    totp_secret_encrypted    BYTEA,
    totp_enabled_at          TIMESTAMPTZ,
    twofa_backup_codes_hash  BYTEA[],
    created_at               TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at               TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at               TIMESTAMPTZ
);
CREATE INDEX users_deleted_at_idx ON users(deleted_at) WHERE deleted_at IS NULL;
CREATE INDEX users_stripe_customer_id_idx ON users(stripe_customer_id) WHERE stripe_customer_id IS NOT NULL;
```

### 0003_sessions.sql

```sql
CREATE TABLE sessions (
    id            UUID PRIMARY KEY,
    user_id       UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash    BYTEA NOT NULL UNIQUE,
    user_agent    TEXT,
    ip            INET,
    last_seen_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at    TIMESTAMPTZ NOT NULL,
    revoked_at    TIMESTAMPTZ
);
CREATE INDEX sessions_token_hash_idx   ON sessions(token_hash);
CREATE INDEX sessions_user_active_idx  ON sessions(user_id) WHERE revoked_at IS NULL;
CREATE INDEX sessions_expires_at_idx   ON sessions(expires_at) WHERE revoked_at IS NULL;
```

### 0004_email_verifications.sql

```sql
CREATE TABLE email_verifications (
    id           UUID PRIMARY KEY,
    user_id      UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash   BYTEA NOT NULL UNIQUE,
    kind         TEXT NOT NULL CHECK (kind IN ('signup','email_change','password_reset')),
    new_email    CITEXT,
    expires_at   TIMESTAMPTZ NOT NULL,
    consumed_at  TIMESTAMPTZ,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK ((kind = 'email_change') = (new_email IS NOT NULL))
);
CREATE INDEX email_ver_user_pending_idx ON email_verifications(user_id, kind)
    WHERE consumed_at IS NULL;
```

### 0005_products.sql

```sql
CREATE TABLE products (
    id                    UUID PRIMARY KEY,
    slug                  TEXT NOT NULL UNIQUE,
    legacy_slug_id        TEXT UNIQUE,
    kind                  TEXT NOT NULL CHECK (kind IN ('indicator','course')),
    name                  TEXT NOT NULL,
    tagline               TEXT NOT NULL,
    description           TEXT NOT NULL,
    price_cents           BIGINT NOT NULL CHECK (price_cents >= 0),
    original_price_cents  BIGINT CHECK (original_price_cents IS NULL OR original_price_cents >= 0),
    active                BOOLEAN NOT NULL DEFAULT TRUE,
    badge                 TEXT,
    rating_value          NUMERIC(3,2) NOT NULL DEFAULT 0.00 CHECK (rating_value BETWEEN 0 AND 5),
    rating_count          INTEGER NOT NULL DEFAULT 0 CHECK (rating_count >= 0),
    highlights            TEXT[] NOT NULL DEFAULT '{}',
    specs_json            JSONB NOT NULL DEFAULT '[]'::jsonb,
    deliverables          TEXT[] NOT NULL DEFAULT '{}',
    requirements          TEXT[] NOT NULL DEFAULT '{}',
    media_poster_color    TEXT NOT NULL,
    media_accent          TEXT NOT NULL,
    stripe_price_id       TEXT UNIQUE,
    created_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at            TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX products_slug_idx        ON products(slug);
CREATE INDEX products_active_kind_idx ON products(kind, active) WHERE active = TRUE;
CREATE INDEX products_stripe_price_id_idx ON products(stripe_price_id) WHERE stripe_price_id IS NOT NULL;
```

### 0006_subscription_plans.sql

```sql
CREATE TABLE subscription_plans (
    id                        UUID PRIMARY KEY,
    slug                      TEXT NOT NULL UNIQUE,
    legacy_slug_id            TEXT UNIQUE,
    name                      TEXT NOT NULL,
    cadence                   TEXT NOT NULL CHECK (cadence IN ('monthly','quarterly','annual')),
    price_cents               BIGINT NOT NULL CHECK (price_cents >= 0),
    monthly_equivalent_cents  BIGINT NOT NULL CHECK (monthly_equivalent_cents >= 0),
    savings_pct               INTEGER NOT NULL DEFAULT 0 CHECK (savings_pct BETWEEN 0 AND 100),
    tagline                   TEXT NOT NULL,
    highlights                TEXT[] NOT NULL DEFAULT '{}',
    featured                  BOOLEAN NOT NULL DEFAULT FALSE,
    badge                     TEXT,
    stripe_price_id           TEXT NOT NULL UNIQUE,
    created_at                TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at                TIMESTAMPTZ NOT NULL DEFAULT now()
);
-- Intent: exactly ONE featured plan across the whole pricing page (the single
-- "most popular" callout in the locked Phase-1 UI). This is deliberate and
-- global, NOT one-featured-per-cadence. If the UI ever wants a highlight per
-- cadence column, this index must change to ON (cadence) WHERE featured.
CREATE UNIQUE INDEX subscription_plans_one_featured_idx
    ON subscription_plans((TRUE)) WHERE featured = TRUE;
```

### 0007_subscriptions.sql

```sql
CREATE TABLE subscriptions (
    id                         UUID PRIMARY KEY,
    user_id                    UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    plan_id                    UUID NOT NULL REFERENCES subscription_plans(id) ON DELETE RESTRICT,
    stripe_subscription_id     TEXT NOT NULL UNIQUE,
    status                     TEXT NOT NULL CHECK (status IN
                                 ('trialing','active','past_due','paused','scheduled_cancel','canceled')),
    cancel_at_period_end       BOOLEAN NOT NULL DEFAULT FALSE,
    current_period_start       TIMESTAMPTZ NOT NULL,
    current_period_end         TIMESTAMPTZ NOT NULL,
    pause_until                TIMESTAMPTZ,
    canceled_at                TIMESTAMPTZ,
    default_payment_method_id  TEXT,
    created_at                 TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at                 TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (current_period_end > current_period_start),
    CHECK ((status = 'canceled') = (canceled_at IS NOT NULL))
);
CREATE INDEX subscriptions_user_status_idx     ON subscriptions(user_id, status);
CREATE INDEX subscriptions_stripe_sub_id_idx   ON subscriptions(stripe_subscription_id);
CREATE INDEX subscriptions_period_end_idx      ON subscriptions(current_period_end)
    WHERE status IN ('active','trialing','past_due');
```

### 0008_orders.sql

```sql
CREATE TABLE orders (
    id                          UUID PRIMARY KEY,
    user_id                     UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    status                      TEXT NOT NULL CHECK (status IN ('pending','paid','refunded','failed')),
    subtotal_cents              BIGINT NOT NULL CHECK (subtotal_cents >= 0),
    tax_cents                   BIGINT NOT NULL DEFAULT 0 CHECK (tax_cents >= 0),
    total_cents                 BIGINT NOT NULL CHECK (total_cents >= 0),
    currency                    TEXT NOT NULL DEFAULT 'usd',
    stripe_checkout_session_id  TEXT UNIQUE,
    stripe_payment_intent_id    TEXT UNIQUE,
    paid_at                     TIMESTAMPTZ,
    refunded_at                 TIMESTAMPTZ,
    cart_snapshot               JSONB NOT NULL DEFAULT '[]'::jsonb,
    expires_at                  TIMESTAMPTZ,
    created_at                  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at                  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX orders_user_created_idx ON orders(user_id, created_at DESC);
CREATE INDEX orders_status_created_idx ON orders(status, created_at DESC)
    WHERE status IN ('pending','failed');
CREATE INDEX orders_stripe_pi_idx ON orders(stripe_payment_intent_id)
    WHERE stripe_payment_intent_id IS NOT NULL;
```

`total = subtotal + tax` is enforced **in code** (`OrdersRepo::create`), not as
a CHECK, so refunds-with-fees and credit-note scenarios don't fight the DB.

### 0009_order_items.sql

```sql
CREATE TABLE order_items (
    id                UUID PRIMARY KEY,
    order_id          UUID NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
    product_id        UUID REFERENCES products(id) ON DELETE RESTRICT,
    plan_id           UUID REFERENCES subscription_plans(id) ON DELETE RESTRICT,
    quantity          INTEGER NOT NULL DEFAULT 1 CHECK (quantity > 0),
    unit_price_cents  BIGINT NOT NULL CHECK (unit_price_cents >= 0),
    line_total_cents  BIGINT NOT NULL CHECK (line_total_cents >= 0),
    name_snapshot     TEXT NOT NULL,
    slug_snapshot     TEXT NOT NULL,
    CHECK ((product_id IS NOT NULL) <> (plan_id IS NOT NULL))
);
CREATE INDEX order_items_order_id_idx ON order_items(order_id);
```

### 0010_invoices.sql

```sql
CREATE TABLE invoices (
    id                 UUID PRIMARY KEY,
    order_id           UUID REFERENCES orders(id) ON DELETE RESTRICT,
    user_id            UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    stripe_invoice_id  TEXT NOT NULL UNIQUE,
    number             TEXT NOT NULL,
    status             TEXT NOT NULL CHECK (status IN ('draft','open','paid','void','uncollectible')),
    amount_cents       BIGINT NOT NULL CHECK (amount_cents >= 0),
    currency           TEXT NOT NULL DEFAULT 'usd',
    invoice_date       TIMESTAMPTZ NOT NULL,
    pdf_r2_key         TEXT,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX invoices_user_date_idx ON invoices(user_id, invoice_date DESC);
```

### 0011_payment_methods.sql

```sql
CREATE TABLE payment_methods (
    id                        UUID PRIMARY KEY,
    user_id                   UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    stripe_payment_method_id  TEXT NOT NULL UNIQUE,
    brand                     TEXT NOT NULL,
    last4                     TEXT NOT NULL CHECK (length(last4) = 4),
    exp_month                 INTEGER NOT NULL CHECK (exp_month BETWEEN 1 AND 12),
    exp_year                  INTEGER NOT NULL CHECK (exp_year BETWEEN 2024 AND 2099),
    is_default                BOOLEAN NOT NULL DEFAULT FALSE,
    created_at                TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX payment_methods_one_default_per_user
    ON payment_methods(user_id) WHERE is_default = TRUE;
CREATE INDEX payment_methods_user_idx ON payment_methods(user_id);
```

### 0012_billing_addresses.sql

```sql
CREATE TABLE billing_addresses (
    user_id        UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    name           TEXT NOT NULL,
    company        TEXT,
    line1          TEXT NOT NULL,
    line2          TEXT,
    city           TEXT NOT NULL,
    state          TEXT,
    postal_code    TEXT NOT NULL,
    country        TEXT NOT NULL CHECK (length(country) = 2),
    tax_id         TEXT,
    billing_email  CITEXT NOT NULL,
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

### 0013_enrollments.sql

```sql
CREATE TABLE enrollments (
    id              UUID PRIMARY KEY,
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    product_id      UUID NOT NULL REFERENCES products(id) ON DELETE RESTRICT,
    cohort          TEXT,
    progress_pct    INTEGER NOT NULL DEFAULT 0 CHECK (progress_pct BETWEEN 0 AND 100),
    last_lesson_id  TEXT,
    source          TEXT NOT NULL CHECK (source IN ('purchase','subscription','manual')),
    source_ref_id   UUID,
    active          BOOLEAN NOT NULL DEFAULT TRUE,
    started_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at    TIMESTAMPTZ
);
CREATE UNIQUE INDEX enrollments_user_product_idx ON enrollments(user_id, product_id);
CREATE INDEX enrollments_user_idx ON enrollments(user_id);
```

### 0014_course_progress.sql

```sql
CREATE TABLE course_progress (
    id                  UUID PRIMARY KEY,
    enrollment_id       UUID NOT NULL REFERENCES enrollments(id) ON DELETE CASCADE,
    lesson_id           TEXT NOT NULL,
    completed_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    notes               TEXT NOT NULL DEFAULT '',
    time_spent_seconds  INTEGER NOT NULL DEFAULT 0 CHECK (time_spent_seconds >= 0),
    payload             JSONB NOT NULL DEFAULT '{}'::jsonb
);
CREATE UNIQUE INDEX course_progress_enrollment_lesson_idx
    ON course_progress(enrollment_id, lesson_id);
```

### 0015_licenses.sql

```sql
CREATE TABLE licenses (
    id                  UUID PRIMARY KEY,
    user_id             UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    product_id          UUID NOT NULL REFERENCES products(id) ON DELETE RESTRICT,
    license_key_hash    BYTEA NOT NULL UNIQUE,
    license_key_prefix  TEXT NOT NULL,
    source              TEXT NOT NULL CHECK (source IN ('purchase','subscription','manual')),
    source_ref_id       UUID,
    issued_at           TIMESTAMPTZ NOT NULL DEFAULT now(),
    revoked_at          TIMESTAMPTZ,
    active              BOOLEAN NOT NULL DEFAULT TRUE,
    download_count      INTEGER NOT NULL DEFAULT 0 CHECK (download_count >= 0),
    last_used_at        TIMESTAMPTZ,
    -- An inactive license must carry a revocation timestamp.
    CHECK (active OR revoked_at IS NOT NULL)
);
CREATE INDEX licenses_user_idx ON licenses(user_id, product_id) WHERE active = TRUE;
```

### 0016_downloads.sql

```sql
CREATE TABLE downloads_catalog (
    id           UUID PRIMARY KEY,
    product_id   UUID NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    platform     TEXT NOT NULL,
    version      TEXT NOT NULL,
    file_r2_key  TEXT NOT NULL,
    sha256       TEXT NOT NULL CHECK (length(sha256) = 64),
    size_bytes   BIGINT NOT NULL CHECK (size_bytes > 0),
    released_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX downloads_catalog_product_idx ON downloads_catalog(product_id, released_at DESC);

CREATE TABLE download_grants (
    id                  UUID PRIMARY KEY,
    user_id             UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    download_id         UUID NOT NULL REFERENCES downloads_catalog(id) ON DELETE CASCADE,
    granted_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_downloaded_at  TIMESTAMPTZ,
    download_count      INTEGER NOT NULL DEFAULT 0 CHECK (download_count >= 0),
    UNIQUE (user_id, download_id)
);
CREATE INDEX download_grants_user_idx ON download_grants(user_id);
```

### 0017_notifications.sql

```sql
CREATE TABLE notification_preferences (
    user_id                UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    product_updates_email  BOOLEAN NOT NULL DEFAULT TRUE,
    product_updates_inapp  BOOLEAN NOT NULL DEFAULT TRUE,
    billing_email          BOOLEAN NOT NULL DEFAULT TRUE,
    billing_inapp          BOOLEAN NOT NULL DEFAULT TRUE,
    course_progress_email  BOOLEAN NOT NULL DEFAULT TRUE,
    course_progress_inapp  BOOLEAN NOT NULL DEFAULT TRUE,
    market_alerts_email    BOOLEAN NOT NULL DEFAULT FALSE,
    market_alerts_inapp    BOOLEAN NOT NULL DEFAULT TRUE,
    marketing_email        BOOLEAN NOT NULL DEFAULT FALSE,
    marketing_inapp        BOOLEAN NOT NULL DEFAULT FALSE,
    dnd_enabled            BOOLEAN NOT NULL DEFAULT FALSE,
    dnd_start              TIME,
    dnd_end                TIME,
    timezone               TEXT NOT NULL DEFAULT 'UTC',
    updated_at             TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK ((dnd_enabled = FALSE) OR (dnd_start IS NOT NULL AND dnd_end IS NOT NULL))
);

CREATE TABLE notifications (
    id          UUID PRIMARY KEY,
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    kind        TEXT NOT NULL,
    title       TEXT NOT NULL,
    body        TEXT NOT NULL,
    source      TEXT NOT NULL,
    read_at     TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX notifications_user_unread_idx
    ON notifications(user_id, created_at DESC) WHERE read_at IS NULL;
CREATE INDEX notifications_user_all_idx ON notifications(user_id, created_at DESC);
```

### 0018_leads_contact.sql

```sql
CREATE TABLE leads (
    id          UUID PRIMARY KEY,
    email       CITEXT NOT NULL,
    source      TEXT NOT NULL DEFAULT 'free-guide',
    ip          INET,
    user_agent  TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX leads_created_at_idx ON leads(created_at DESC);
CREATE INDEX leads_email_idx      ON leads(email);

CREATE TABLE contact_messages (
    id          UUID PRIMARY KEY,
    name        TEXT NOT NULL,
    email       CITEXT NOT NULL,
    subject     TEXT NOT NULL,
    body        TEXT NOT NULL,
    ip          INET,
    status      TEXT NOT NULL DEFAULT 'new' CHECK (status IN ('new','read','archived','spam')),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX contact_messages_status_created_idx ON contact_messages(status, created_at DESC);
```

### 0019_stripe_events.sql

```sql
-- Two concerns, deliberately kept separable on one table:
--   (a) idempotency  -> event_id PK + ON CONFLICT DO NOTHING
--   (b) lifecycle    -> processed_at lets a received-but-unprocessed event
--                       be found and re-driven; a crash between insert and
--                       dispatch is no longer invisible.
-- payload is for replay/debug ONLY and is NOT load-bearing for dedupe. It is
-- nullable so a retention job can null it out without losing the dedupe row.
CREATE TABLE stripe_events (
    event_id          TEXT PRIMARY KEY,         -- Stripe's evt_… id IS the PK
    event_type        TEXT NOT NULL,
    payload           JSONB,                    -- nullable: replay/debug only
    received_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    processed_at      TIMESTAMPTZ,
    attempts          INTEGER NOT NULL DEFAULT 0 CHECK (attempts >= 0),
    processing_error  TEXT
);
-- Drives the reconciliation sweep (§8.3): everything received but not processed.
CREATE INDEX stripe_events_unprocessed_idx ON stripe_events(received_at)
    WHERE processed_at IS NULL;
CREATE INDEX stripe_events_kind_idx ON stripe_events(event_type, received_at DESC);
```

### 0020_audit_log.sql

```sql
CREATE TABLE audit_log (
    id             UUID PRIMARY KEY,
    actor_user_id  UUID REFERENCES users(id) ON DELETE SET NULL,
    action         TEXT NOT NULL,
    target_kind    TEXT NOT NULL,
    target_id      TEXT NOT NULL,
    metadata       JSONB NOT NULL DEFAULT '{}'::jsonb,
    ip             INET,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX audit_log_actor_created_idx ON audit_log(actor_user_id, created_at DESC)
    WHERE actor_user_id IS NOT NULL;
CREATE INDEX audit_log_target_idx ON audit_log(target_kind, target_id, created_at DESC);
```

### 0021_background_jobs.sql

```sql
CREATE TABLE background_jobs (
    id            UUID PRIMARY KEY,
    kind          TEXT NOT NULL,
    payload       JSONB NOT NULL DEFAULT '{}'::jsonb,
    status        TEXT NOT NULL DEFAULT 'queued'
                    CHECK (status IN ('queued','running','succeeded','failed','dead')),
    attempts      INTEGER NOT NULL DEFAULT 0 CHECK (attempts >= 0),
    max_attempts  INTEGER NOT NULL DEFAULT 5,
    run_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    started_at    TIMESTAMPTZ,
    finished_at   TIMESTAMPTZ,
    last_error    TEXT,
    schedule_key  TEXT,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (kind, schedule_key)
);
CREATE INDEX background_jobs_claim_idx ON background_jobs(status, run_at) WHERE status = 'queued';
CREATE INDEX background_jobs_kind_idx  ON background_jobs(kind, created_at DESC);
```

**That is the complete migration set: 0001 through 0021. There is no 0022.**

---

## 5. Money — `common::money::Money`

The newtype's whole purpose is to make silent money errors impossible. This
revision removes the last place a silent error could hide — `mul_bps`'s old
`clamp` — and makes the serde guard symmetric.

```rust
// crates/common/src/money.rs
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sqlx::{Decode, Encode, Postgres, Type, postgres::PgValueRef};
use std::{fmt, ops::{Add, Sub, Neg}};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Money(i64);

const MAX_JS_SAFE: i64 = (1i64 << 53) - 1;
const MIN_JS_SAFE: i64 = -((1i64 << 53) - 1);

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum MoneyError {
    #[error("money arithmetic overflowed i64")]
    Overflow,
    #[error("money value {0} is outside the JS-safe integer range")]
    OutsideJsSafeRange(i64),
}

impl Money {
    pub const ZERO: Money = Money(0);

    #[inline] pub const fn from_cents(c: i64) -> Self { Money(c) }
    #[inline] pub const fn cents(self) -> i64 { self.0 }

    /// Multiply by a basis-point factor (1 bps = 0.01%). Widens to i128, then
    /// narrows back with a checked conversion. NEVER saturates: an out-of-range
    /// result is an explicit `Err`, not a clamped lie.
    pub fn mul_bps(self, bps: i32) -> Result<Money, MoneyError> {
        let prod: i128 = (self.0 as i128) * (bps as i128);
        let scaled: i128 = prod / 10_000;
        i64::try_from(scaled).map(Money).map_err(|_| MoneyError::Overflow)
    }

    /// Multiply by a whole-percent factor. Defined in terms of `mul_bps`.
    pub fn mul_pct(self, pct: i32) -> Result<Money, MoneyError> {
        let bps = pct.checked_mul(100).ok_or(MoneyError::Overflow)?;
        self.mul_bps(bps)
    }

    pub fn checked_add(self, r: Money) -> Option<Money> { self.0.checked_add(r.0).map(Money) }
    pub fn checked_sub(self, r: Money) -> Option<Money> { self.0.checked_sub(r.0).map(Money) }

    /// Reject values that would lose precision once they cross into JS.
    fn ensure_js_safe(v: i64) -> Result<i64, MoneyError> {
        if (MIN_JS_SAFE..=MAX_JS_SAFE).contains(&v) {
            Ok(v)
        } else {
            Err(MoneyError::OutsideJsSafeRange(v))
        }
    }
}

// Add/Sub/Neg use checked ops and panic on overflow in debug, wrap in release
// — but in practice all money arithmetic at call sites goes through
// `checked_add`/`checked_sub`. These operator impls exist for ergonomics in
// test code and total-rollup loops where inputs are already bounded.
impl Add for Money { type Output = Money; fn add(self, r: Money) -> Money { Money(self.0 + r.0) } }
impl Sub for Money { type Output = Money; fn sub(self, r: Money) -> Money { Money(self.0 - r.0) } }
impl Neg for Money { type Output = Money; fn neg(self) -> Money { Money(-self.0) } }

// Deliberately NO Mul<i64>, NO Mul<Money>. Quantity-times-price goes through
// an explicit helper that widens to i128 first and returns Result.

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let neg = self.0 < 0;
        let abs = self.0.unsigned_abs();
        write!(f, "{}${}.{:02}", if neg { "-" } else { "" }, abs / 100, abs % 100)
    }
}

// SYMMETRIC JS-safety guard: reject out-of-range on BOTH directions.
impl Serialize for Money {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        Money::ensure_js_safe(self.0).map_err(serde::ser::Error::custom)?;
        s.serialize_i64(self.0)
    }
}
impl<'de> Deserialize<'de> for Money {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let v = i64::deserialize(d)?;
        Money::ensure_js_safe(v).map(Money).map_err(serde::de::Error::custom)
    }
}

impl Type<Postgres> for Money { fn type_info() -> sqlx::postgres::PgTypeInfo { <i64 as Type<Postgres>>::type_info() } }
impl<'q> Encode<'q, Postgres> for Money {
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer)
        -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>>
    { <i64 as Encode<Postgres>>::encode_by_ref(&self.0, buf) }
}
impl<'r> Decode<'r, Postgres> for Money {
    fn decode(v: PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>>
    { Ok(Money(<i64 as Decode<Postgres>>::decode(v)?)) }
}
```

Every call site that previously did `money.mul_bps(x)` now handles a `Result`.
Tax and discount computation in `OrdersRepo::create` propagates `MoneyError`
into `AppError::Internal` — a money overflow is a bug, not a user error, and
must surface loudly.

---

## 6. Axum app skeleton

`AppState` carries an `alerts` sink — the webhook handler and `AppError`
reporting both depend on it, so it is a first-class field, not an
afterthought referenced by code that wouldn't compile.

```rust
// crates/http/src/app.rs
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub stripe: Arc<dyn StripeApi>,
    pub r2: Arc<dyn ObjectStore>,
    pub mailer: Arc<dyn Mailer>,
    pub jobs: Arc<JobEnqueuer>,
    pub limiter: Arc<dyn RateLimiter>,
    pub alerts: Arc<dyn AlertSink>,          // <-- declared, see §8.3 / §14
    pub hash_semaphore: Arc<Semaphore>,      // Semaphore(8), see §7.2
    pub config: Arc<Config>,
    pub users: UsersRepo,
    pub sessions: SessionsRepo,
    pub orders: OrdersRepo,
    pub subs: SubscriptionsRepo,
    pub products: ProductsRepo,
    pub plans: PlansRepo,
    pub invoices: InvoicesRepo,
    pub licenses: LicensesRepo,
    pub enrollments: EnrollmentsRepo,
    pub notifications: NotificationsRepo,
    pub leads: LeadsRepo,
    pub contact: ContactRepo,
    pub audit: AuditRepo,
    pub stripe_events: StripeEventsRepo,
}

/// Side-channel for "this should never fail, but it did" events. Implemented
/// by a Resend-backed pager in prod and a recording mock in tests.
#[async_trait]
pub trait AlertSink: Send + Sync {
    /// Fire-and-forget: spawns its own task, never blocks the caller, never
    /// returns an error (an alert that can fail isn't an alert).
    fn fire_async(&self, kind: AlertKind);
}

#[derive(Debug, Clone)]
pub enum AlertKind {
    WebhookHandlerFailed { event_id: String, error: String },
    JobDeadLettered { kind: String, job_id: String, error: String },
    ReadinessDegraded { check: &'static str },
}

pub fn build_router(state: AppState) -> Router {
    let public = Router::new()
        .route("/healthz", get(health::live))
        .route("/readyz",  get(health::ready))
        .route("/v1/auth/signup",          post(auth::signup))
        .route("/v1/auth/login",           post(auth::login))
        .route("/v1/auth/login/totp",      post(auth::login_totp))
        .route("/v1/auth/logout",          post(auth::logout))
        .route("/v1/auth/forgot-password", post(auth::forgot))
        .route("/v1/auth/reset-password",  post(auth::reset))
        .route("/v1/auth/verify-email",    post(auth::verify_email))
        .route("/v1/public/leads",         post(public::lead_capture))
        .route("/v1/public/contact",       post(public::contact))
        .route("/v1/public/products",      get(public::list_products))
        .route("/v1/public/plans",         get(public::list_plans))
        .route("/v1/webhooks/stripe",      post(webhooks::stripe)
            .layer(DefaultBodyLimit::max(256 * 1024)))   // override 1 MiB to 256 KiB
        .layer(rate_limit::public(state.limiter.clone()));

    let authed = Router::new()
        .nest("/v1/me",            me::router())
        .nest("/v1/sessions",      sessions::router())
        .nest("/v1/subscription",  subscription::router())
        .nest("/v1/billing",       billing::router())
        .nest("/v1/courses",       courses::router())
        .nest("/v1/indicators",    indicators::router())
        .nest("/v1/notifications", notifications::router())
        .nest("/v1/checkout",      checkout::router())
        .nest("/v1/downloads",     downloads::router())
        .route_layer(middleware::from_fn_with_state(state.clone(), auth::require_auth))
        .layer(rate_limit::session(state.limiter.clone()));

    let admin = Router::new()
        .nest("/v1/admin", admin::router())
        .route_layer(middleware::from_fn_with_state(state.clone(), auth::require_admin));

    // NOTE: no CorsLayer. Rust has no public URL; the BFF is the only client,
    // it is server-side, and it authenticates with X-Service-Token (§1.1).
    Router::new()
        .merge(public).merge(authed).merge(admin)
        .layer(
            ServiceBuilder::new()
                .layer(CatchPanicLayer::new())
                .layer(SetRequestIdLayer::x_request_id(MakeUuid))
                .layer(PropagateRequestIdLayer::x_request_id())
                .layer(TraceLayer::new_for_http()
                    .make_span_with(trace::make_span)
                    .on_response(trace::on_response))
                .layer(TimeoutLayer::new(Duration::from_secs(15)))
                .layer(RequestBodyLimitLayer::new(1024 * 1024))
                .layer(middleware::from_fn_with_state(state.clone(), service_token::verify))
                .layer(CompressionLayer::new().gzip(true))
        )
        .with_state(state)
}
```

`service_token::verify` rejects any request whose `x-service-token` header is
absent or not constant-time-equal to `config.service_token`, and whose source
IP is outside `config.service_token_ip_allowlist`. It runs on every route,
including `/healthz` — the load balancer health probe is given the token too.

---

## 7. Auth — sessions, Argon2id, TOTP

### 7.1 Session cookie

- Name: `tfx_session`
- Format: `v1.<b64url(session_id)>.<b64url(token)>.<b64url(hmac_sha256(key, "v1." + id + "." + token))>`
- Token: 32 random bytes from `OsRng`.
- DB lookup: parse → verify HMAC (constant-time) → `sha256(token)` → SELECT
  WHERE `token_hash = $1 AND revoked_at IS NULL AND expires_at > now()`.
- TTL: 30-day rolling. `last_seen_at` updated at most once per 60 s via a
  `moka` write-through cache.
- Attrs: `HttpOnly; Secure; SameSite=Lax; Path=/`.

### 7.2 Argon2id wrapper — keyed pepper, hash AND verify

The pepper is passed as Argon2's `secret` key, not concatenated. This means the
hash string in the DB does **not** encode the pepper, and verification must
reconstruct the same keyed `Argon2` instance. Both halves are shown so the
asymmetry that bit the earlier draft cannot recur.

```rust
use argon2::{Argon2, Algorithm, Version, Params, PasswordHash, PasswordHasher,
             PasswordVerifier, password_hash::{SaltString, rand_core::OsRng}};
use secrecy::{ExposeSecret, SecretString};

/// Build the Argon2id instance with the pepper as the keyed `secret`.
/// Params: m = 64 MiB, t = 3, p = 1.
fn argon2_with_pepper(pepper: &[u8]) -> Result<Argon2<'_>, AppError> {
    let params = Params::new(64 * 1024, 3, 1, None)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("argon2 params: {e}")))?;
    Argon2::new_with_secret(pepper, Algorithm::Argon2id, Version::V0x13, params)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("argon2 secret: {e}")))
}

/// Hash a password. Runs on a blocking thread; the caller holds a permit from
/// AppState::hash_semaphore (Semaphore(8)) before invoking this.
pub async fn hash_password(pw: SecretString, pepper: SecretVec<u8>) -> Result<String, AppError> {
    tokio::task::spawn_blocking(move || {
        let argon = argon2_with_pepper(pepper.expose_secret())?;
        let salt = SaltString::generate(&mut OsRng);
        argon
            .hash_password(pw.expose_secret().as_bytes(), &salt)
            .map(|h| h.to_string())
            .map_err(|e| AppError::Internal(anyhow::anyhow!("argon2 hash: {e}")))
    })
    .await
    .map_err(|_| AppError::Internal(anyhow::anyhow!("argon2 hash task join failed")))?
}

/// Verify a password against a stored hash. MUST rebuild the SAME keyed
/// Argon2 instance — the pepper is not in `stored`, so a plain
/// `PasswordHash::verify_password(&[default])` would always reject.
pub async fn verify_password(
    pw: SecretString,
    stored: String,
    pepper: SecretVec<u8>,
) -> Result<bool, AppError> {
    tokio::task::spawn_blocking(move || {
        let argon = argon2_with_pepper(pepper.expose_secret())?;
        let parsed = PasswordHash::new(&stored)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("argon2 parse: {e}")))?;
        match argon.verify_password(pw.expose_secret().as_bytes(), &parsed) {
            Ok(()) => Ok(true),
            Err(argon2::password_hash::Error::Password) => Ok(false),
            Err(e) => Err(AppError::Internal(anyhow::anyhow!("argon2 verify: {e}"))),
        }
    })
    .await
    .map_err(|_| AppError::Internal(anyhow::anyhow!("argon2 verify task join failed")))?
}
```

`AppState::hash_semaphore` is a `Semaphore(8)` — every `hash_password` /
`verify_password` call acquires a permit first, capping concurrent 64-MiB
hashes. Set `tokio::runtime::Builder::max_blocking_threads(32)`.

### 7.3 TOTP

- 20-byte secret, encrypted with `XChaCha20-Poly1305` using `AUTH_TOTP_KEY`.
- Stored in `users.totp_secret_encrypted` (NULL until confirmed).
- **Skew pinned in code.** The `TOTP` value is constructed with an explicit
  `skew = 1` (current step ± 1, ≈90 s total window), SHA-1, 6 digits, 30 s
  period — all passed as literals, never relying on a `totp-rs` default:

  ```rust
  let totp = totp_rs::TOTP::new(
      totp_rs::Algorithm::SHA1,
      6,                       // digits
      1,                       // skew  <-- explicit, pinned
      30,                      // period seconds
      secret_bytes,
  )?;
  ```

- Enable: generate secret → return `otpauth://` URI + QR (data URL).
- Confirm: user submits 6-digit code → decrypt + verify → set `totp_enabled_at`.
- Disable: requires current password + current TOTP code.
- Backup codes: 10 codes, each 10 chars (Crockford base32), Argon2id-hashed
  (same keyed-pepper instance as passwords), stored as
  `users.twofa_backup_codes_hash BYTEA[]`. Mark consumed by replacing with
  all-zero bytes (preserves array length so we can show "X/10 remaining").

### 7.4 Extractor

```rust
pub struct AuthSession {
    pub user_id: UserId,
    pub session_id: SessionId,
    pub role: Role,
}

#[async_trait]
impl FromRequestParts<AppState> for AuthSession {
    type Rejection = AppError;
    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, AppError> {
        let cookies = parts.extract::<TypedHeader<Cookie>>().await
            .map_err(|_| AppError::Unauthorized)?;
        let raw = cookies.get("tfx_session").ok_or(AppError::Unauthorized)?;
        let parsed = SessionCookie::verify(raw, &state.config.cookie_key)?;
        state.sessions.load(parsed).await
    }
}
```

### 7.5 Revoke all

```sql
UPDATE sessions SET revoked_at = now()
WHERE user_id = $1 AND revoked_at IS NULL AND id <> $2;   -- $2 = current session (keep)
```

---

## 8. Stripe integration

### 8.1 Crate `stripe-client`

Thin wrapper around `async-stripe`. Returns **domain types** (not Stripe
types) so we can swap providers later. Method list:

- `get_or_create_customer_for_user(user, idem)` — idempotent on `user_id`.
- `create_checkout_session_for_products(args, idem)`
- `create_checkout_session_for_subscription(args, idem)`
- `create_customer_portal_session(customer_ref, return_url, idem)`
- `cancel_subscription_at_period_end(sub_id, idem)`
- `resume_subscription(sub_id, idem)`
- `pause_subscription(sub_id, idem)`
- `update_subscription_default_payment_method(sub_id, pm_id, idem)`
- `list_invoices(customer_ref, limit)`
- `download_invoice_pdf(invoice_id)` → `Bytes`
- `refund_payment_intent(pi_id, idem)`
- `retrieve_subscription(sub_id)` — used by the reconciliation cron.

### 8.2 Idempotency keys

Derived in `domain::idempotency::derive_key`:

```
tf:{base32(sha256(prefix || user_id || action || nonce))[..24]}
```

- `prefix` = `"tradeflex-prod"` or `"tradeflex-stg"`.
- `action` = `"checkout"`, `"refund"`, `"cancel_sub"`, etc.
- `nonce` per action:
  - checkout: cart hash (deterministic — retried checkout collapses).
  - refund: `order_id`.
  - cancel: `subscription_id`.

### 8.3 Webhook receiver — insert-unprocessed → dispatch → mark-processed

This is the highest-severity fix in the revision. The earlier draft inserted
the dedupe row and, on a successful insert, dispatched — but never recorded
`processed_at`. If the process died **after** the insert and **before**
`dispatch_stripe_event` finished, every later delivery of that event saw the
row already present and skipped it. A crashed `checkout.session.completed`
meant a paid customer with no entitlement and nothing to catch it.

The corrected flow:

1. Verify the Stripe signature against raw bytes.
2. `INSERT … ON CONFLICT (event_id) DO NOTHING` — the row goes in with
   `processed_at = NULL`.
3. If the row already existed **and** is already `processed_at IS NOT NULL`,
   it's a true duplicate → 200, done.
4. If the row already existed but `processed_at IS NULL`, a previous attempt
   crashed mid-dispatch → fall through and re-dispatch (handlers are
   idempotent — §22 rule 4).
5. Run `dispatch_stripe_event` **inside a DB transaction** that, on success,
   also sets `processed_at = now()` and bumps `attempts`. Commit atomically.
6. On handler error: bump `attempts`, write `processing_error`, leave
   `processed_at` NULL, fire an alert. Still return 200 to Stripe (its retries
   are harmless and the reconciliation sweep is the real backstop).

```rust
pub async fn stripe_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<StatusCode, WebhookError> {
    let sig = headers.get("stripe-signature")
        .and_then(|v| v.to_str().ok())
        .ok_or(WebhookError::MissingSignature)?;

    let event = stripe::Webhook::construct_event(
        std::str::from_utf8(&body).map_err(|_| WebhookError::Malformed)?,
        sig,
        state.config.stripe_webhook_secret.expose_secret(),
    ).map_err(|_| WebhookError::InvalidSignature)?;

    // Step 2: claim the event. Returns the row's processed_at if it pre-existed.
    let claim = sqlx::query!(
        r#"
        INSERT INTO stripe_events (event_id, event_type, payload, received_at)
        VALUES ($1, $2, $3, now())
        ON CONFLICT (event_id) DO UPDATE SET event_id = stripe_events.event_id
        RETURNING (xmax = 0) AS "freshly_inserted!", processed_at
        "#,
        event.id.as_str(),
        event.type_.as_str(),
        serde_json::to_value(&event).ok(),
    )
    .fetch_one(&state.db)
    .await
    .map_err(WebhookError::Db)?;

    // Step 3: genuine duplicate of an already-completed event.
    if !claim.freshly_inserted && claim.processed_at.is_some() {
        tracing::info!(event_id = %event.id, "duplicate stripe event already processed, skipping");
        return Ok(StatusCode::OK);
    }
    // else: fresh insert OR a prior crashed attempt — process (handlers idempotent).

    // Step 5 + 6: dispatch inside a tx; mark processed atomically on success.
    let mut tx = state.db.begin().await.map_err(WebhookError::Db)?;
    match dispatch_stripe_event(&state, &mut tx, &event).await {
        Ok(()) => {
            sqlx::query!(
                r#"UPDATE stripe_events
                   SET processed_at = now(), attempts = attempts + 1, processing_error = NULL
                   WHERE event_id = $1"#,
                event.id.as_str(),
            ).execute(&mut *tx).await.map_err(WebhookError::Db)?;
            tx.commit().await.map_err(WebhookError::Db)?;
        }
        Err(e) => {
            tx.rollback().await.map_err(WebhookError::Db)?;
            // Separate, non-tx write so the error/attempts survive the rollback.
            let _ = sqlx::query!(
                r#"UPDATE stripe_events
                   SET attempts = attempts + 1, processing_error = $2
                   WHERE event_id = $1"#,
                event.id.as_str(), e.to_string(),
            ).execute(&state.db).await;
            tracing::error!(event_id = %event.id, error = ?e, "stripe handler failed");
            state.alerts.fire_async(AlertKind::WebhookHandlerFailed {
                event_id: event.id.to_string(),
                error: e.to_string(),
            });
        }
    }
    Ok(StatusCode::OK)   // always 200 unless signature/body invalid
}
```

Body **must** be `Bytes`, not `Json<_>` — Stripe signs the raw bytes.

**Reconciliation sweep — covers one-time checkouts, not just subscriptions.**
A worker cron job `reconcile_stripe_events` (every 5 min) selects
`stripe_events WHERE processed_at IS NULL AND received_at < now() - interval '2 minutes'`,
and for each one re-fetches the event from Stripe (`GET /v1/events/{id}`) and
re-runs the dispatch+mark flow above. This is the true backstop for a webhook
delivery that crashed mid-flight — including a one-time
`checkout.session.completed`, which the subscription-only nightly
`refresh_subscription_status` job would never have caught. After 10 failed
attempts the sweep stops retrying that row and escalates via `AlertSink`.

### 8.4 Event handlers

`dispatch_stripe_event` takes `&mut Transaction` so the critical DB writes and
the `processed_at` stamp commit or roll back as one unit.

| Event | Inline DB tx | Side-effect jobs |
|---|---|---|
| `checkout.session.completed` | orders→paid; invoices insert; subscriptions upsert (if mode=subscription); enrollments / licenses insert; audit_log | `send_email(receipt)`, `fetch_and_store_invoice_pdf` |
| `invoice.paid` (renewal) | subscriptions.current_period_end refresh; invoices insert | `send_email(subscription_renewed)`, `fetch_and_store_invoice_pdf` |
| `customer.subscription.updated` | subscriptions upsert (status, cancel_at_period_end, plan_id) | `send_email(cancellation_scheduled)` or `send_email(plan_changed)` if relevant fields changed |
| `customer.subscription.deleted` | subscriptions.status='canceled'; **revoke subscription-source entitlements right here** (the event fires exactly at period end — no self-scheduled job) | `send_email(subscription_canceled)` |
| `invoice.payment_failed` | subscriptions.status='past_due'; invoices insert | `send_email(payment_failed)` |
| `charge.refunded` | orders.status='refunded'; revoke order-derived entitlements; audit_log | `send_email(refund_issued)` |
| `customer.subscription.trial_will_end` | — | `send_email(trial_ending_soon)` |

**No self-scheduled `revoke_subscription_entitlements` job.** The earlier draft
scheduled a future `background_jobs` row to fire at `current_period_end`. But
Stripe already emits `customer.subscription.deleted` at that exact instant;
running our own delayed job duplicates Stripe's clock and drifts from it.
Revocation is done inline in the `deleted` handler. (`run_at` on
`background_jobs` is still used — e.g. `expire_pending_order` — just not to
re-implement Stripe's own timing.)

**Always 200 to Stripe** unless signature is invalid or body is malformed.
Handler errors fire an alert; the reconciliation sweep re-drives them.

### 8.5 Entitlements

- License key format: `TF-{kind}-{4x4-Crockford-base32}` (24 chars, dashes).
- Shown plaintext **once** at issuance (receipt email + dashboard "copy now").
- Stored as `Argon2id(plaintext)` hash + first-8 prefix.
- Revocation rules:
  - `source='purchase'` → only revoked on `charge.refunded` for the parent order.
  - `source='subscription'` → revoked inline by the
    `customer.subscription.deleted` handler.
  - Manual admin grants never revoked by webhooks.

### 8.6 Customer Portal

`POST /v1/billing/portal` → fetch `customer_ref` → `create_customer_portal_session`
with `return_url=$PUBLIC_SITE_URL + /dashboard/billing` → return `{url}`.
SvelteKit redirects.

Stripe Dashboard config (documented in `docs/stripe-setup.md`):

- Allowed actions: update PM, view invoices, **cancel at period end**, plan
  switch among the three cadences.
- **Disallow immediate cancellation** — `cancel_at_period_end` is cleaner to
  mirror.

---

## 9. R2 storage

### 9.1 Crate `r2-client`

```rust
#[async_trait]
pub trait ObjectStore: Send + Sync {
    async fn presigned_get(&self, key: &str, ttl: Duration) -> Result<Url, StoreError>;
    async fn presigned_put(&self, key: &str, ttl: Duration, content_type: &str) -> Result<Url, StoreError>;
    async fn put_object(&self, key: &str, bytes: Bytes, content_type: &str) -> Result<(), StoreError>;
    async fn delete(&self, key: &str) -> Result<(), StoreError>;
    async fn head(&self, key: &str) -> Result<ObjectMeta, StoreError>;
}

pub struct R2Client { s3: aws_sdk_s3::Client, bucket: String }
```

R2 quirks: `region("auto")`, `force_path_style(true)`, SigV4 signing. Validate
during the spike that presigned URLs work end-to-end; if `aws-sdk-s3` produces
URLs R2 rejects, fall back to `rusty-s3` for presigning specifically.

### 9.2 Key conventions (`crates/domain/src/r2_keys.rs`)

- `downloads/{product_id}/{version}/{filename}` — indicator binaries
- `invoices/{user_id}/{invoice_id}.pdf`
- `exports/{user_id}/{job_id}.json.gz`
- `courses/{course_id}/{lesson_id}.mp4`

### 9.3 Download flow — entitlement check and audit in one transaction

The presign and the `audit_log` insert run inside the **same DB transaction**.
The earlier draft listed them as loose sequential steps, which allowed a URL to
be handed out with no audit row if the insert failed. Order:

```
GET /v1/downloads/{download_id}/url
  → AuthSession
  → BEGIN
  → SELECT downloads_catalog.* WHERE id = $1                       FOR SHARE
  → entitlement check:
       EXISTS(licenses    WHERE user_id=$me AND product_id=$prod AND active)
    OR EXISTS(enrollments WHERE user_id=$me AND product_id=$prod AND active)
  → INSERT audit_log (action='download.presigned', target=$download_id)
  → COMMIT                          -- audit row is durable before URL is minted
  → r2.presigned_get(file_r2_key, 300s)   -- network call, OUTSIDE the tx
  → enqueue job 'bump_download_counter' { download_id, user_id }
  → 200 { url, expires_at }
```

The presign itself is a pure local SigV4 computation against R2's endpoint and
is done after `COMMIT` so a slow/failed network call can't hold a DB
transaction open. The invariant that matters — "no URL without an audit row" —
holds because the audit insert is committed before the presigned URL exists.

### 9.4 Invoice PDFs

`checkout.session.completed` and `invoice.paid` enqueue
`fetch_and_store_invoice_pdf { invoice_id, user_id }`. Worker:

1. `stripe.download_invoice_pdf(invoice_id)`
2. `r2.put_object("invoices/{user}/{inv}.pdf", bytes, "application/pdf")`
3. `UPDATE invoices SET pdf_r2_key = $1 WHERE id = $2`

`GET /v1/billing/invoices/{id}/pdf-url` checks ownership → presigned 300 s.

### 9.5 Data exports

`POST /v1/me/export-data` is rate-limited to 1 per 24 h per user. Returns 202.

Worker `generate_data_export`:

1. `SET TRANSACTION ISOLATION LEVEL REPEATABLE READ`
2. SELECT * from every user-owned table where `user_id = $1`
3. Assemble + gzip
4. `r2.put_object("exports/{user}/{job}.json.gz", bytes, "application/gzip")`
5. Presigned GET with `ttl = 24h` (the one deliberate deviation from 300 s,
   documented at the call site)
6. Enqueue `send_email(data_export_ready, link)`

---

## 10. Email — `mailer`

```rust
#[async_trait]
pub trait Mailer: Send + Sync {
    async fn send(&self, msg: OutgoingEmail) -> Result<MessageId, MailerError>;
}

pub struct ResendMailer { client: resend_rs::Resend, from: String }
```

Templates (each `name.html` + `name.txt`, MJML source compiled at build time):

- `welcome`
- `verify_email`
- `password_reset`
- `email_change_confirm`
- `receipt`
- `subscription_renewed`
- `subscription_canceled`
- `trial_ending_soon`
- `payment_failed`
- `plan_changed`
- `refund_issued`
- `data_export_ready`
- `account_deletion_scheduled`
- `account_deletion_completed`

**Every send goes through the queue** as `background_jobs.kind=send_email`
with rendered HTML+text in the payload. Hard `assert!(payload.len() < 256*1024)`
at enqueue.

`test-support` provides `RecordingMailer { sent: Mutex<Vec<OutgoingEmail>> }`.

---

## 11. Background worker — `bins/worker`

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg = Config::from_env()?;
    observability::init(&cfg)?;          // JSON stdout + optional OTLP, see §15
    let db = PgPool::connect(cfg.database_url.expose_secret()).await?;
    sqlx::migrate!("../../migrations").run(&db).await?;

    let stripe = StripeClient::new(cfg.stripe_secret_key.clone(), "tf");
    let r2     = R2Client::new(cfg.r2_endpoint.clone(), cfg.r2_access_key.clone(),
                               cfg.r2_secret_key.clone(), cfg.r2_bucket.clone());
    let mailer = ResendMailer::new(cfg.resend_api_key.clone(), cfg.mail_from.clone());

    let shutdown = CancellationToken::new();
    register_signal_handler(shutdown.clone());

    let scheduler = tokio::spawn(scheduler_loop(db.clone(), shutdown.clone()));
    let workers: Vec<_> = (0..cfg.worker_concurrency)
        .map(|_| tokio::spawn(worker_loop(db.clone(), stripe.clone(), r2.clone(),
                                          mailer.clone(), shutdown.clone())))
        .collect();
    for w in workers { let _ = w.await; }
    let _ = scheduler.await;
    Ok(())
}
```

Claim query:

```sql
SELECT id, kind, payload, attempts, max_attempts
FROM background_jobs
WHERE status = 'queued' AND run_at <= now()
ORDER BY run_at
LIMIT 1
FOR UPDATE SKIP LOCKED
```

Retry backoff `[60s, 5m, 30m, 2h, 12h]`, dead-letter after 5 (fires
`AlertKind::JobDeadLettered`).

**Job kinds:**

- `send_email` — Resend.
- `fetch_and_store_invoice_pdf` — Stripe → R2.
- `generate_data_export` — DB → R2 → email.
- `expire_pending_order` — flip pending orders older than 1 h to `failed`
  (uses `run_at` legitimately — no external signal exists for this).
- `bump_download_counter`
- `dispatch_notification` — in-app notification fanout.
- `reconcile_stripe_events` (cron, every 5 min) — re-drives any
  `stripe_events` row with `processed_at IS NULL` older than 2 min; the
  backstop for crashed webhook dispatch including one-time checkouts (§8.3).
- `gc_expired_sessions` (cron hourly)
- `gc_expired_email_verifications` (cron hourly)
- `refresh_subscription_status` (cron nightly 03:00 UTC) — sanity-pulls Stripe
  state for any subscription with `current_period_end < now() + 7d`; a
  secondary safety net specifically for subscription drift.

There is **no** `revoke_subscription_entitlements` job — revocation is inline
in the `customer.subscription.deleted` handler (§8.4).

Scheduler enqueues cron jobs idempotently:

```sql
INSERT INTO background_jobs (id, kind, payload, schedule_key)
VALUES ($1, $2, $3, $4)
ON CONFLICT (kind, schedule_key) DO NOTHING
```

---

## 12. Rate limiting

Trait in `domain`:

```rust
#[async_trait]
pub trait RateLimiter: Send + Sync {
    async fn check(&self, key: RateKey, bucket: Bucket) -> Result<(), RateLimited>;
}
```

In-process impl uses `governor::RateLimiter<RateKey, DashMapStateStore>`.

Buckets:

| Surface | Quota | Key |
|---|---|---|
| Public read | 60/min | IP |
| Signup | 5/hour | IP |
| Login | 10/min | (IP, normalized_email) — both must pass |
| Forgot password | 3/hour | IP |
| Reset password | 5/15min | IP |
| Lead capture | 5/min | IP |
| Contact | 3/min | IP |
| Authed read | 300/min | session |
| Authed write | 60/min | session |
| Admin | 600/min | session |
| Download presign | 20/min | user |
| Data export | 1/24h | user |
| Webhooks | unlimited | (signature gate) |

---

## 13. Config & secrets

Loaded once at startup via `figment` (env + optional `.env` in dev).

| Group | Var | Type |
|---|---|---|
| DB | `DATABASE_URL` | Secret |
| DB | `DATABASE_MAX_CONN` (default 20), `DATABASE_MIN_CONN` (default 2) | int |
| Stripe | `STRIPE_SECRET_KEY` | Secret |
| Stripe | `STRIPE_WEBHOOK_SECRET` | Secret |
| Stripe | `STRIPE_API_VERSION` | string |
| R2 | `R2_ACCOUNT_ID`, `R2_BUCKET`, `R2_ENDPOINT` | string |
| R2 | `R2_ACCESS_KEY_ID`, `R2_SECRET_ACCESS_KEY` | Secret |
| Resend | `RESEND_API_KEY` | Secret |
| Resend | `MAIL_FROM`, `MAIL_REPLY_TO` | string |
| Auth | `AUTH_COOKIE_KEY` (32-byte b64) | Secret |
| Auth | `AUTH_PASSWORD_PEPPER` | Secret |
| Auth | `AUTH_TOTP_KEY` (32-byte b64) | Secret |
| Auth | `AUTH_SESSION_TTL_DAYS` (default 30) | int |
| Auth | `AUTH_COOKIE_DOMAIN` | string |
| BFF | `SERVICE_TOKEN` | Secret |
| BFF | `SERVICE_TOKEN_IP_ALLOWLIST` | CIDR list |
| Obs | `RUST_LOG`, `METRICS_BIND` | string |
| Obs | `OTEL_EXPORTER_OTLP_ENDPOINT` (optional — OTLP layer installed only if set) | string |
| Obs | `OTEL_SERVICE_NAME` (default `tradeflex-api` / `tradeflex-worker`) | string |
| Bind | `BIND_ADDR` (default `0.0.0.0:8080`), `ENV` | string |

There is no `CORS_ALLOWED_ORIGINS` — Rust has no public/browser-facing surface
(§1.1). Failing to parse any required var aborts boot with a one-line error.
`OTEL_EXPORTER_OTLP_ENDPOINT` is genuinely optional: unset → no OTLP exporter,
stdout JSON tracing still runs.

---

## 14. Error model

```rust
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("validation: {0}")]     Validation(String),
    #[error("unauthorized")]        Unauthorized,
    #[error("forbidden")]           Forbidden,
    #[error("not found")]           NotFound,
    #[error("conflict: {0}")]       Conflict(String),
    #[error("rate limited")]        RateLimited { retry_after_secs: u32 },
    #[error("external: {service}")] External { service: &'static str, #[source] source: anyhow::Error },
    #[error("internal")]            Internal(#[from] anyhow::Error),
}
```

`IntoResponse` maps each variant to an HTTP status + JSON body
`{ "error": { "code", "message" }, "request_id" }`. `Internal`/`External` never
echo the source string to the client; on either, the handler may fire an
`AlertSink` event (`AlertKind` defined in §6) — `AlertSink` is the single,
declared mechanism for "should-never-happen" reporting and is reachable from
both `AppError` mapping and the webhook handler.

`MoneyError` (§5) converts into `AppError::Internal` — a money overflow is a
bug, surfaced loudly, never a clamped value returned to a caller.

---

## 15. Observability

`crates/observability` owns setup; both `api` and `worker` call
`observability::init(&cfg)` once at startup.

- `tracing-subscriber` JSON layer to stdout — **always installed.**
- `tracing-opentelemetry` layer exporting OTLP spans to
  `OTEL_EXPORTER_OTLP_ENDPOINT` — **installed only if that var is set.** Local
  dev with no collector simply runs stdout-only. Service name from
  `OTEL_SERVICE_NAME`. This is the real OpenTelemetry wiring; the env var is
  not decorative.
- Span: `http_request{method, route, request_id}` — `route` is the matched
  route, never the raw URI.
- Metrics on `:9090/metrics`:
  - Counters: `http_requests_total{route, method, status}`,
    `auth_attempts_total{kind, outcome}`,
    `jobs_processed_total{kind, outcome}`,
    `stripe_webhook_events_total{event_type, outcome}`.
  - Histograms: `http_request_duration_seconds{route, method}`,
    `db_query_duration_seconds{op}`,
    `argon2_hash_duration_seconds`,
    `stripe_call_duration_seconds{action}`.
  - **Hard rule:** every label has ≤ 20 distinct values; raw user input is
    never a label.
- `/healthz`: 200 unconditionally if the process is alive.
- `/readyz`: 200 only if DB `SELECT 1` < 100 ms AND Stripe `GET /v1/balance`
  (30 s cached) returned 2xx last attempt. Body shape:
  `{ checks: { db: "ok"|"fail", stripe: "ok"|"fail" } }`. A failing check fires
  `AlertKind::ReadinessDegraded`.
- Request ID propagated via `SetRequestIdLayer` + a `tokio::task_local`.

---

## 16. Testing

### 16.1 Pyramid

- **Unit** (pure, in-crate): Money math (including `mul_bps` overflow → `Err`,
  and serde rejection on both directions), idempotency-key derivation, license
  formatter/parser, password hash + **verify round-trip with the keyed
  pepper**, entitlement decision tree.
- **Integration** (per-test ephemeral Postgres): **one strategy, committed —
  the `testcontainers` schema-per-test approach.** Each test run spins one
  Postgres container; each test gets its own freshly-created schema (or
  database) with migrations applied, and the test's pool is opened against it.
  The earlier draft also floated `CREATE DATABASE … WITH TEMPLATE`, but
  Postgres refuses to clone a template while any session is connected to it,
  which races against a live connection pool under parallel `cargo test`.
  Pick one and only one: it is `testcontainers` + per-test schema. If a
  template-clone is ever wanted for speed, the template pool must be opened
  with `min_connections = 0` and explicitly closed before any clone — but that
  is not the chosen path here.
- **Stripe E2E** (`stripe-e2e` feature, `--test-threads=1`): real test keys,
  Stripe CLI listener forwarded to a local port, `stripe trigger` drives
  webhook fixtures, assertions poll the DB with a timeout.
- **R2 E2E**: LocalStack for per-PR runs, real R2 in a per-CI-run bucket on
  `main` push.
- **Webhook signature + crash-recovery**: recorded payload fixtures +
  signature; assert valid accepts, tampered rejects, old-timestamp rejects,
  replay deduplicates, **and a simulated crash between insert and dispatch
  leaves `processed_at` NULL so the reconciliation sweep re-drives it.**
- **Property tests** (`proptest`): Money arithmetic, license-key uniqueness
  over 1 M generations, idempotency-key determinism.
- **Load tests** (`oha` binary): hot endpoints, p95 < 100 ms at 100 RPS on
  Railway standard plan, baseline checked in.
- **Coverage gate**: `cargo llvm-cov` ≥ 80 % on `domain` + `http`.

### 16.2 MCP tool requirement (CLAUDE.md hard rule)

- Every `.rs` edit → `mcp__rust-analyzer__diagnostics` (replaces `cargo check`),
  `code_action` / `rename` for refactors, `cargo fmt`, `cargo clippy
  --all-targets -- -D warnings` as the final gate.
- Every `.svelte` edit (BFF adapter side) → `mcp__svelte__list-sections` →
  `get-documentation` → `svelte-autofixer`. (BFF adapters are mostly
  `+page.server.ts`, which is `.ts` not `.svelte`; the rule applies whenever
  any `.svelte` file is touched.)

### 16.3 Hard-evidence checklist (CLAUDE.md habit #2)

Every feature ships only when `docs/launch-evidence/{feature}.md` shows green:

- **Signup**: `curl POST /v1/auth/signup` → 201, DB row exists, verify email
  in Resend test inbox, `Set-Cookie` has `HttpOnly; Secure; SameSite=Lax`.
- **Password verify round-trip**: hash with the real pepper, verify the same
  password → true, verify a wrong password → false, verify after restarting
  the process (pepper reloaded from env) → still true.
- **Login lock-out**: 11 wrong-password attempts from one IP → 429 with
  `Retry-After`.
- **Subscription purchase**: SvelteKit "Subscribe Monthly" → Stripe Checkout
  → `4242 4242 4242 4242` → redirect → within 10 s,
  `subscriptions.status='active'` + `current_period_end ≈ now+30d`,
  receipt email in Resend, invoice in `invoices`, presigned PDF link works.
- **Renewal**: `stripe trigger invoice.paid` → `current_period_end` extends,
  renewal email arrives.
- **Refund**: Stripe Dashboard refund → within 30 s, `orders.status='refunded'`,
  `enrollments.active=false` for that order, refund email arrives.
- **Cancel via portal**: portal cancel → `subscriptions.cancel_at_period_end=
  true` immediately → at period end (`stripe trigger
  customer.subscription.deleted`), `status='canceled'` and subscription-source
  enrollments `active=false` **set inline by the handler**, cancellation email
  arrives.
- **Indicator download**: `GET /v1/downloads/{id}/url` with a valid session →
  200 + URL + `expires_at` ≈ 5 min → `curl URL` returns file bytes with the
  right `Content-Length` → flip a byte in the URL path → 403 from R2 → confirm
  an `audit_log` row exists for the presign.
- **2FA**: enable → scan QR → confirm code → 10 backup codes shown once →
  `users.totp_enabled_at IS NOT NULL` → logout → wrong TOTP → 401 → right
  TOTP → 200; a code from the *previous* 30 s step is still accepted (skew=1);
  a code from two steps ago is rejected; a backup code consumes a slot.
- **Data export**: `POST /v1/me/export-data` → 202 → within 2 min, R2 has
  the gz, an email with a 24 h presigned link arrives, `curl LINK | gunzip | jq`
  shows the user's rows.
- **Webhook idempotency**: `stripe trigger checkout.session.completed`
  twice with the same event id → second delivery sees `processed_at` set and
  logs "duplicate stripe event already processed, skipping"; zero
  double-enrollment.
- **Webhook crash-recovery**: kill the process between the `stripe_events`
  insert and the dispatch commit → row has `processed_at IS NULL` →
  `reconcile_stripe_events` re-drives it within 5 min → exactly one enrollment
  row exists, `processed_at` now set.
- **`FOR UPDATE` race**: deliberately race two webhook deliveries (same event
  id, different replicas) → exactly one enrollment row exists.
- **EXPLAIN ANALYZE**: every index in §4 is used by its named query path.
- **`/readyz`**: stop Postgres → 503 within 1 s → restart → 200 within 5 s.

---

## 17. CI/CD

`.github/workflows/backend.yml`:

```yaml
name: backend
on:
  push: { paths: ['backend/**', '.github/workflows/backend.yml'] }
  pull_request: { paths: ['backend/**'] }

jobs:
  fmt:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with: { components: rustfmt }
      - run: cargo fmt --all --check
        working-directory: backend

  clippy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with: { components: clippy }
      - uses: Swatinem/rust-cache@v2
      - run: cargo clippy --workspace --all-targets -- -D warnings
        working-directory: backend

  sqlx-prepare:
    runs-on: ubuntu-latest
    services:
      postgres: { image: postgres:16, env: { POSTGRES_PASSWORD: postgres }, ports: ['5432:5432'] }
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo install sqlx-cli --no-default-features --features postgres
      - run: cargo sqlx prepare --workspace --check
        working-directory: backend
        env: { DATABASE_URL: postgres://postgres:postgres@localhost:5432/postgres }

  test:
    runs-on: ubuntu-latest
    # testcontainers manages Postgres + LocalStack from inside the test process
    # (Docker is available on ubuntu-latest); no `services:` block needed.
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo test --workspace --no-fail-fast
        working-directory: backend

  stripe-e2e:
    runs-on: ubuntu-latest
    needs: test
    if: github.ref == 'refs/heads/main' || contains(github.event.pull_request.labels.*.name, 'stripe-e2e')
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - name: install stripe-cli
        run: curl -sSL https://github.com/stripe/stripe-cli/releases/latest/download/stripe_linux_x86_64.tar.gz | tar xz -C /usr/local/bin stripe
      - run: cargo test --workspace --features stripe-e2e -- --test-threads=1
        working-directory: backend
        env:
          STRIPE_TEST_SK: ${{ secrets.STRIPE_TEST_SK }}
          STRIPE_TEST_WHSEC: ${{ secrets.STRIPE_TEST_WHSEC }}

  coverage:
    runs-on: ubuntu-latest
    needs: test
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with: { components: llvm-tools-preview }
      - run: cargo install cargo-llvm-cov
      - run: cargo llvm-cov --workspace --fail-under-lines 80 -p domain -p http
        working-directory: backend

  deny:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: EmbarkStudios/cargo-deny-action@v2
        with: { manifest-path: backend/Cargo.toml }
```

PR cannot merge red. Required checks: fmt, clippy, sqlx-prepare, test,
coverage, deny.

---

## 18. Local dev — `justfile`

```just
up:
    docker compose -f backend/docker-compose.dev.yml up -d

down:
    docker compose -f backend/docker-compose.dev.yml down -v

migrate:
    cd backend && cargo sqlx migrate run

dump-catalog:
    pnpm exec tsx scripts/dump-catalog.ts > backend/seeds/catalog.json

seed: dump-catalog
    cd backend && cargo run -p seeder

dev:
    cd backend && cargo watch -x 'run -p api'

worker:
    cd backend && cargo run -p worker

stripe-listen:
    stripe listen --forward-to localhost:8080/v1/webhooks/stripe

test:
    cd backend && cargo test --workspace

test-e2e:
    cd backend && cargo test --workspace --features stripe-e2e -- --test-threads=1

load-test:
    oha -z 60s -c 50 http://localhost:8080/v1/public/products

clippy:
    cd backend && cargo clippy --workspace --all-targets -- -D warnings

fmt:
    cd backend && cargo fmt --all
```

---

## 19. Full BFF endpoint inventory

The Rust API surface that SvelteKit `+page.server.ts` files will call.

### Public (`/v1/public/**`, `/v1/auth/**`, `/v1/webhooks/**`)

- `POST /v1/auth/signup` → 201, sets cookie, enqueues welcome + verify emails.
- `POST /v1/auth/login` → 200, sets cookie. With TOTP: returns `{step: 'totp', tx_id}`; client follows up with `POST /v1/auth/login/totp { tx_id, code }`.
- `POST /v1/auth/login/totp` → 200, sets cookie.
- `POST /v1/auth/logout` → 204, revokes the current session.
- `POST /v1/auth/forgot-password` → 202, always 202 (no email enumeration).
- `POST /v1/auth/reset-password` → 204.
- `POST /v1/auth/verify-email` → 204.
- `GET  /v1/public/products` (filter by `kind=indicator|course`) → list.
- `GET  /v1/public/products/{slug}` → detail.
- `GET  /v1/public/plans` → list.
- `POST /v1/public/leads` → 201.
- `POST /v1/public/contact` → 201.
- `POST /v1/webhooks/stripe` → 200/400.

### Authenticated user (`/v1/me/**`, `/v1/subscription/**`, etc.)

- `GET  /v1/me` → user + active sub + entitlement summary.
- `PATCH /v1/me` → name, headline, timezone, language.
- `POST /v1/me/change-email` → enqueue verify; pending until confirmed.
- `POST /v1/me/change-password` → requires the current password.
- `POST /v1/me/2fa/enable` → returns otpauth URI + QR data URL.
- `POST /v1/me/2fa/confirm` → first valid TOTP; returns 10 backup codes.
- `POST /v1/me/2fa/disable` → requires password + TOTP.
- `POST /v1/me/2fa/backup-codes/regenerate` → returns 10 fresh codes.
- `GET  /v1/me/sessions` → list active sessions (device, location, last_seen).
- `DELETE /v1/me/sessions/{id}` → revoke one.
- `POST /v1/me/sessions/revoke-others` → revoke all except the current.
- `POST /v1/me/export-data` → 202 with job_id.
- `POST /v1/me/delete-account` → schedule deletion in 30 d; cancellable.
- `GET  /v1/subscription` → active sub + scheduled changes.
- `POST /v1/subscription/change-plan` → schedules at next renewal via Stripe.
- `POST /v1/subscription/pause` → 1–3 months.
- `POST /v1/subscription/resume` → cancels a scheduled cancel/pause.
- `POST /v1/subscription/cancel` → at period end.
- `GET  /v1/billing/orders` → paginated history.
- `GET  /v1/billing/invoices` → list.
- `GET  /v1/billing/invoices/{id}/pdf-url` → presigned R2.
- `GET  /v1/billing/payment-methods` → list cards.
- `POST /v1/billing/payment-methods/setup-intent` → Stripe SetupIntent.
- `PATCH /v1/billing/payment-methods/{id}/default`
- `DELETE /v1/billing/payment-methods/{id}`
- `GET  /v1/billing/address` / `PATCH /v1/billing/address`
- `POST /v1/billing/portal` → Stripe Customer Portal URL.
- `POST /v1/checkout` → cart → Stripe Checkout session URL.
- `GET  /v1/courses` → enrollments (dashboard view).
- `GET  /v1/courses/{slug}` → course player state (modules, lessons, progress).
- `POST /v1/courses/{slug}/progress` → lesson complete, time spent.
- `PUT  /v1/courses/{slug}/lessons/{lesson_id}/notes`
- `GET  /v1/indicators` → owned licenses + downloads.
- `GET  /v1/indicators/{slug}/key` → returns the plaintext license **once at issuance**, otherwise the prefix only.
- `GET  /v1/indicators/{slug}/downloads` → catalog of platform versions.
- `GET  /v1/downloads/{download_id}/url` → presigned R2.
- `GET  /v1/downloads` → all downloadable artifacts for this user.
- `GET  /v1/notifications` → in-app feed (paginated).
- `PATCH /v1/notifications/{id}/read`
- `POST /v1/notifications/mark-all-read`
- `GET  /v1/notifications/preferences` / `PATCH /v1/notifications/preferences`

### Admin (`/v1/admin/**`, requires `role='admin'`)

- `GET  /v1/admin/stats` → MRR, active subs, new signups, conversion (sparklines).
- `GET  /v1/admin/revenue-chart?range=12mo`
- `GET  /v1/admin/top-products?limit=10`
- `GET  /v1/admin/recent-orders?limit=10`
- `GET  /v1/admin/recent-leads?limit=10`
- `GET  /v1/admin/recent-messages?limit=10`
- `GET  /v1/admin/leads?limit=200`
- `GET  /v1/admin/leads.csv` (CSV stream)
- `GET  /v1/admin/messages?status=new&limit=200`
- `PATCH /v1/admin/messages/{id}` → status (read/archived/spam).
- `GET  /v1/admin/products` / `POST /v1/admin/products` /
  `GET /v1/admin/products/{id}` / `PATCH /v1/admin/products/{id}` /
  `DELETE /v1/admin/products/{id}` / `POST /v1/admin/products/{id}/duplicate`
- `GET  /v1/admin/plans` / CRUD
- `GET  /v1/admin/orders?status=&q=`
- `GET  /v1/admin/orders/{id}`
- `POST /v1/admin/orders/{id}/refund` → triggers a Stripe refund.
- `GET  /v1/admin/customers?q=`
- `GET  /v1/admin/customers/{id}` → orders + sub + contact history.
- `POST /v1/admin/customers/{id}/grant-entitlement` → manual enrollment / license.

---

## 20. BFF integration (SvelteKit side)

For each Rust endpoint above, a thin `+page.server.ts` (or `+server.ts`)
adapter in `src/routes/**` calls Rust via `fetch` with:

- **`X-Service-Token: <SERVICE_TOKEN>`** — the one canonical service-auth
  header (§1.1), env-loaded, never exposed to the client. Not
  `Authorization: Service …`.
- `X-Request-Id: <uuid>` propagated back to the browser.
- `Cookie: tfx_session=…` forwarded from the incoming browser request.
- `Set-Cookie` forwarded back from Rust to the browser.

The existing form-action shape (`actions: { default, capture, subscribe }`)
stays — handlers internally `await fetch(rustUrl, …)` and translate Rust
4xx/5xx into Svelte `fail()` / `redirect()`.

A small `src/lib/server/rust-client.ts` centralizes:

- Base URL from `env.RUST_API_BASE_URL`.
- `X-Service-Token` header injection.
- Error mapping (Rust JSON `{ error: { code, message } }` → typed throw).
- Cookie pass-through.

Phase 1's local SQLite + Drizzle migrations get retired in the cutover PR:

- `drizzle/`, `drizzle.config.ts`, `src/lib/server/db/*`, `db:*` scripts in
  `package.json` are deleted (final commit).
- `pnpm-lock.yaml` drops `@libsql/client`, `drizzle-orm`, `drizzle-kit`.
- An ESLint rule blocking `@libsql/client` imports is added.

---

## 21. Phased rollout (PR-by-PR)

Each PR ships independently mergeable, behind a feature flag where the
SvelteKit BFF picks Rust vs. legacy.

| # | PR | Verification |
|---|---|---|
| 1 | Workspace skeleton, `common::money` (Result-returning `mul_*`, symmetric serde guard), `AppError` + `AlertSink`, `Config`, `observability` crate, `/healthz`, `/readyz`, CI green | `cargo run -p api`; curl healthz; Money overflow unit test; CI green |
| 2 | Migrations 0001–0021; `storage::pool`; `seeder` binary; catalog snapshot dump script | `just migrate && just seed`; psql verifies row counts |
| 3 | `auth` — signup/login/logout, sessions, Argon2id keyed pepper (hash + verify); `service_token` middleware (`x-service-token`) | Evidence §16.3 signup, password verify round-trip, login lockout |
| 4 | `me` — profile, change-email, change-password, sessions list/revoke | Evidence checklist; manual sessions revoke |
| 5 | TOTP enable/confirm/disable + backup codes (skew=1 pinned) | Evidence checklist 2FA incl. skew boundary cases |
| 6 | `public` — products/plans/leads/contact; rate limiter; SvelteKit BFF rewrites for `/free-guide`, `/contact` | Evidence + EXPLAIN on `leads_created_at_idx` |
| 7 | `stripe-client` + `checkout` endpoint + webhook receiver (signature verify, insert-unprocessed → dispatch → mark-processed, dispatch=noop) | Stripe CLI signature test; webhook 200; crash-recovery test |
| 8 | `subscription` endpoints + `checkout.session.completed` handler + entitlements (purchase → license/enrollment) + `reconcile_stripe_events` cron | Evidence checklist subscription purchase + webhook crash-recovery |
| 9 | Customer Portal endpoint + `invoice.paid` + `customer.subscription.*` handlers (revocation inline on `deleted`) + renewal flow | Evidence checklist renewal + cancel |
| 10 | `r2-client` + invoice PDF fetch job + presigned URLs for invoices | Evidence checklist PDF download + tamper rejection |
| 11 | `indicators` endpoints + license key issuance + downloads catalog + download grants (presign + audit in one tx) | Evidence checklist indicator download incl. audit row |
| 12 | `courses` endpoints + progress + notes | UI walks the course player end-to-end |
| 13 | `notifications` (prefs + feed) + `dispatch_notification` job | Evidence: send event, see in feed, mark read |
| 14 | `admin` KPI + leads/messages + products/plans CRUD + orders/customers | UI manual walkthrough |
| 15 | Refund flow (admin trigger + `charge.refunded` handler + revoke + email) | Evidence checklist refund |
| 16 | Data export job + `account_deletion_scheduled` flow | Evidence checklist export + delete |
| 17 | Drizzle/SQLite removal PR + production env wiring (Railway secrets) + Vercel BFF env vars | All previous evidence reruns against Railway |

Cutover PR (#17) flips a feature flag `USE_RUST_BACKEND=true` in SvelteKit
and deletes Phase 1 server code.

---

## 22. Hard rules — CLAUDE.md compliance

Restated explicitly so they're impossible to miss:

1. **Every `.rs` edit** → rust-analyzer MCP diagnostics + `cargo fmt` +
   `cargo clippy --all-targets -- -D warnings`. No exceptions.
2. **Every `.svelte` edit** (BFF adapter side) → `mcp__svelte__list-sections`
   → `get-documentation` → `svelte-autofixer`.
3. **Money i64 everywhere.** Postgres `BIGINT`. Rust `Money`. No floats.
   Multiplication via `mul_bps` / `mul_pct` only — and both return `Result`;
   an overflow is an `Err`, never a saturated value.
4. **`SELECT … FOR UPDATE`** on every state-machine transition: order
   processing, license issuance, sub pause/cancel, job claim. Stripe webhook
   dedupe uses `INSERT … ON CONFLICT` (the correct primitive for that case);
   webhook handlers are idempotent and safe to re-run, which the
   reconciliation sweep depends on.
5. **Argon2id / bcrypt** wrapped in `tokio::task::spawn_blocking`, pepper
   supplied as the keyed `secret`, and **every hash path has a matching,
   tested verify path**.
6. **External clients** (Stripe, R2, Resend, anything `reqwest`) always set
   `.timeout()` AND `.connect_timeout()`.
7. **Prometheus labels bounded** — at most ~20 distinct values per label;
   raw user input is never a label.
8. **Audit logs** via the `record_admin_action` helper, written in the same
   transaction as the action they describe — never best-effort after a
   `tracing::warn!` shrug.
9. **Cite the rule** in every commit message touching auth, RBAC, audit,
   migration, or money.
10. **Runtime evidence** before "done": `EXPLAIN ANALYZE`, `curl :PORT/metrics
    | grep`, Stripe Dashboard screenshot, Playwright spec, browser DevTools.
11. **Re-read `git diff --staged`** before commit. Check cached state, comment
    drift, label cardinality, optimistic-UI loading flags.
12. **No `--no-verify`**, no skipping hooks, no destructive git on `main`.
13. **No mocks for the DB** in integration tests — real Postgres via
    `testcontainers` only.
14. **Always create new commits** on hook failure, never `--amend`.
15. **Migrations are forward-only** and contain only one change-set each.

---

## 23. Open risks (acknowledged, accepted)

1. `async-stripe` lags Stripe API versions historically. Keep the wrapper
   thin; ready to drop to `reqwest` per-endpoint.
2. `governor` in-process rate limits don't span replicas. Acceptable at 1
   Railway instance. The trait abstraction prevents rewrites when we scale.
3. Argon2id at m=64 MiB can starve a small box on burst logins. Mitigated by
   `Semaphore(8)` + `max_blocking_threads(32)`.
4. R2 presigned URL TTLs mean revocation isn't instant. 5 min is acceptable
   for indicators/invoices; 24 h for exports is documented at the call site.
5. Soft-deleted users still own rows. Every join needs `WHERE deleted_at
   IS NULL`. Risk of ghost rows; mitigation is a code-review focus.
6. `uuid v7` leaks creation timestamps in public URLs. Phase 3: add a
   `public_ref` opaque short ID for customer-visible resources.
7. `stripe_events.payload` JSONB grows ~6 KB/event. Because `payload` is now
   nullable and **not** load-bearing for dedupe (§8.3), the Phase 3 retention
   job simply `UPDATE … SET payload = NULL` for processed events older than
   90 d — the dedupe row (`event_id` PK + `processed_at`) is tiny and kept
   indefinitely.
8. The background-job queue caps at ~50 jobs/sec on Postgres row locks.
   Documented exit: Redis/SQS when we cross it.
9. Cookie-domain pinning means Vercel preview deploys are separate session
   worlds. Accepted.
10. The worker is a single point of failure. Every job kind must be
    idempotent / safe-to-rerun. Document this in every new job kind's PR.
11. The reconciliation sweep re-fetches events from Stripe; a prolonged Stripe
    outage delays processing but never drops events (the `stripe_events` row
    persists with `processed_at IS NULL` until it succeeds). Accepted.

---

## 24. Critical files

- `/Users/billyribeiro/tradeflex/new-beginning/backend/Cargo.toml`
- `/Users/billyribeiro/tradeflex/new-beginning/backend/migrations/0001_extensions.sql` (through `0021_background_jobs.sql`)
- `/Users/billyribeiro/tradeflex/new-beginning/backend/crates/http/src/app.rs`
- `/Users/billyribeiro/tradeflex/new-beginning/backend/crates/http/src/middleware/auth.rs`
- `/Users/billyribeiro/tradeflex/new-beginning/backend/crates/http/src/middleware/service_token.rs`
- `/Users/billyribeiro/tradeflex/new-beginning/backend/crates/http/src/routes/webhooks/stripe.rs`
- `/Users/billyribeiro/tradeflex/new-beginning/backend/crates/common/src/money.rs`
- `/Users/billyribeiro/tradeflex/new-beginning/backend/crates/common/src/error.rs`
- `/Users/billyribeiro/tradeflex/new-beginning/backend/crates/observability/src/lib.rs`
- `/Users/billyribeiro/tradeflex/new-beginning/backend/bins/api/src/main.rs`
- `/Users/billyribeiro/tradeflex/new-beginning/backend/bins/worker/src/main.rs`
- `/Users/billyribeiro/tradeflex/new-beginning/backend/bins/seeder/src/main.rs`
- `/Users/billyribeiro/tradeflex/new-beginning/scripts/dump-catalog.ts`
- `/Users/billyribeiro/tradeflex/new-beginning/src/lib/server/rust-client.ts`
- `/Users/billyribeiro/tradeflex/new-beginning/.github/workflows/backend.yml`
- `/Users/billyribeiro/tradeflex/new-beginning/STACK.md` (update post-cutover)
- `/Users/billyribeiro/tradeflex/new-beginning/CHANGELOG.md` (entry per PR)

---

## 25. End-to-end verification (the bar for "Phase 2 done")

All of the following must be green, with screenshots / log excerpts in
`docs/launch-evidence/`:

1. Fresh user → signup → email verify → login → enable 2FA → logout → login
   with TOTP → the cookie is `HttpOnly Secure SameSite=Lax`.
2. Buy `Revolution Ranger` indicator via Stripe Checkout (test card) →
   `orders.status=paid`, license issued, receipt email arrives, the license
   key shows on `/dashboard/indicators` exactly once, the indicator download
   works (signed R2 URL), a tampered URL → 403, an `audit_log` row exists.
3. Subscribe to `Day Trading Monthly` → `subscriptions.status=active` →
   wait (or `stripe trigger`) for renewal → invoice in `/dashboard/billing`,
   PDF downloads, renewal email arrives.
4. Cancel from the Customer Portal → `cancel_at_period_end=true` →
   `stripe trigger customer.subscription.deleted` → subscription-source
   enrollments revoked inline by the handler, cancellation email arrives.
5. An admin refunds an order → the entitlement is revoked, refund email arrives.
6. An admin views KPIs → MRR / active subs match a hand-computed query.
7. Lead capture from `/free-guide` → row in `leads`, `EXPLAIN ANALYZE`
   confirms `leads_created_at_idx` is used on `/admin/leads`.
8. The contact form is rate-limited at 3/min from one IP → the 4th returns 429.
9. Login brute-forced 11× from one IP → 429.
10. `/readyz` returns 503 when Postgres is down, recovers within 5 s on
    restart.
11. CI pipeline green: fmt, clippy, sqlx-prepare, test, stripe-e2e,
    coverage ≥ 80 %, cargo-deny clean.
12. `cargo llvm-cov` ≥ 80 % on `domain` + `http`.
13. `oha` load test: p95 < 100 ms at 100 RPS on `/v1/public/products`,
    `/v1/me`, `/v1/courses`.
14. Webhook idempotency: `stripe trigger checkout.session.completed` twice
    with the same event id → exactly one enrollment row.
15. Webhook crash-recovery: process killed between the `stripe_events` insert
    and the dispatch commit → `reconcile_stripe_events` re-drives the event →
    exactly one enrollment row, `processed_at` set.
16. Data export: `POST /v1/me/export-data` → email with a 24 h presigned link
    → `curl LINK | gunzip | jq` reproduces every user-owned row.
17. Money: a unit test proves `Money::from_cents(i64::MAX).mul_pct(200)` returns
    `Err(MoneyError::Overflow)` — never a saturated value — and that
    deserializing a JSON number above `2^53-1` into `Money` is rejected.

---

## 26. Revision 2 — flaw-fix changelog

Every item below is a correction to Revision 1. Listed so reviewers can audit
each decision against the original.

1. **`AppState.alerts` / `AlertSink` / `AlertKind` now declared.** §6 defines
   the trait and enum; §8.3 and §14 reference a type that exists. R1's webhook
   code called `state.alerts` against a struct with no such field.
2. **Argon2id pepper is keyed, not concatenated.** §1.2 / §7.2 use
   `Argon2::new_with_secret`; the matching `verify_password` path is shown in
   full. R1 concatenated the pepper onto the password bytes and never showed
   verify, leaving a hash/verify asymmetry.
3. **TOTP skew pinned in code.** §1.2 / §7.3 set `skew = 1` as an explicit
   constructor literal with the validity window stated, instead of relying on a
   crate default.
4. **`Money` serde guard is symmetric.** §1.5 / §5 reject out-of-JS-range
   values on both `Serialize` and `Deserialize`. R1 only guarded serialize.
5. **`mul_bps` / `mul_pct` return `Result`, no silent saturation.** §1.5 / §5.
   R1 clamped to `i64::MAX`, reintroducing the silent-money-error the newtype
   exists to prevent.
6. **Service-token auth has one canonical name.** §1.1 / §6 / §20 all use
   `X-Service-Token` (`x-service-token` on the wire). R1 used three spellings.
7. **`CorsLayer` removed.** §1.1 / §6 / §13 — Rust has no public/browser-facing
   surface, so CORS was dead code. The `cors` `tower-http` feature and
   `CORS_ALLOWED_ORIGINS` env var are gone.
8. **Webhook handler: insert-unprocessed → dispatch → mark-processed.** §8.3
   now writes `processed_at` inside the dispatch transaction; a crash mid-flight
   leaves the row re-drivable. R1 never set `processed_at`, so a crash silently
   dropped the event.
9. **Reconciliation sweep covers one-time checkouts.** §8.3 / §11 add
   `reconcile_stripe_events` (every 5 min) re-driving any unprocessed
   `stripe_events` row. R1's only backstop was the subscription-only nightly
   job.
10. **No self-scheduled `revoke_subscription_entitlements` job.** §1.7 / §8.4 /
    §11 — revocation is inline in the `customer.subscription.deleted` handler,
    which Stripe already fires at period end. R1 duplicated Stripe's clock.
11. **Integration-test DB strategy is singular and committed.** §16.1 / §17 —
    `testcontainers` schema-per-test only; the racy `CREATE DATABASE … WITH
    TEMPLATE` path is explicitly rejected. R1 presented both as interchangeable.
12. **`oha` correctly placed.** §3 — `oha` is a CLI binary invoked from the
    `justfile`/CI, not a fake `[workspace.dev-dependencies]` library entry
    (that manifest table does not exist either; per-crate `[dev-dependencies]`
    is used).
13. **`askama` version + `askama_axum` removal flagged.** §3 — pinned to a
    release whose Axum support is in-crate, with an explicit
    version-verification rule before PR #1.
14. **`pgcrypto` purpose documented.** §1.6 / §4 `0001` — kept for
    `digest()`/`gen_salt()`, explicitly *not* for UUIDs (PKs are Rust
    `now_v7()`); no DB-side UUID default exists anywhere.
15. **Download flow: presign + audit in one transaction.** §9.3 — the
    `audit_log` insert commits before the presigned URL is minted, closing the
    "URL handed out, no audit row" gap. Plus the migration count is corrected
    everywhere to **0001–0021** (no `0022`), and OpenTelemetry is now actually
    wired (§1.9 / §3 / §15) rather than gestured at by a lone `OTEL_SERVICE_NAME`
    var.

---

**This is the plan, Revision 2. Implementation begins at PR #1 (workspace
skeleton).**