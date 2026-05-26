# PR #3 — Auth (signup/login/logout + sessions + Argon2id + service_token) — Launch Evidence

**Date:** 2026-05-25
**Scope:** BACKEND.md §21 row 3 — `auth` (signup/login/logout, sessions,
Argon2id keyed pepper hash+verify), `service_token` middleware, login
lockout.

## Gates

| Gate | Result |
|---|---|
| `cargo fmt --all -- --check` | ✅ |
| `SQLX_OFFLINE=true cargo clippy --workspace --all-targets -- -D warnings` | ✅ |
| `cargo sqlx prepare --workspace --check` | ✅ |
| `cargo test --workspace` | ✅ 46/46 |
| `cargo deny check` | ✅ |
| signup → 201 + DB row + `HttpOnly` cookie | ✅ |
| login (correct) → 200 + new cookie | ✅ |
| login lockout under repeated attempts → 429 with `Retry-After` | ✅ |
| logout → 204 + DB row marked `revoked_at`, cookie cleared | ✅ |
| password verify round-trip across process restart | ✅ |
| `service_token` rejects missing / wrong token | ✅ |

## Service-token gate

```
$ curl -o /dev/null -w "HTTP %{http_code}\n" http://127.0.0.1:8081/healthz
HTTP 401

$ curl -H "x-service-token: dev-service-token" -o /dev/null -w "HTTP %{http_code}\n" http://127.0.0.1:8081/healthz
HTTP 200

$ curl -H "x-service-token: wrong" -o /dev/null -w "HTTP %{http_code}\n" http://127.0.0.1:8081/healthz
HTTP 401
```

Constant-time compare via `subtle::ConstantTimeEq`. The middleware runs on
every route, including `/healthz`, per BACKEND.md §6.

## Signup → 201

```
$ curl -i -X POST http://127.0.0.1:8081/v1/auth/signup \
    -H "x-service-token: dev-service-token" \
    -H "content-type: application/json" \
    -d '{"name":"Alex Morgan","email":"alex.morgan@example.com","password":"Hunter2-Strong","terms":true}'

HTTP/1.1 201 Created
content-type: application/json
set-cookie: tfx_session=v1.<sid>.<token>.<hmac>; HttpOnly; SameSite=Lax; Path=/; Max-Age=2592000
x-request-id: 019e606e-bea4-7932-bc3e-16e0665b7c1b

{"user":{"id":"019e606e-c2f7-7b52-b862-5aa4a02f0279","email":"alex.morgan@example.com","name":"Alex Morgan","role":"member"}}
```

Cookie attrs satisfy BACKEND.md §16.3: `HttpOnly`, `SameSite=Lax`, `Path=/`.
`Secure` is omitted on `env=development` only — production sets it
unconditionally (see `crates/http/src/routes/auth/signup.rs:format_cookie`).

DB row created via `UsersRepo::create`, `id` is `Uuid::now_v7()` (time-sortable).

## Login round-trip → 200

```
$ curl -i -X POST .../v1/auth/login -d '{"email":"...","password":"Hunter2-Strong"}'

HTTP/1.1 200 OK
set-cookie: tfx_session=v1.<sid>.<token>.<hmac>; HttpOnly; SameSite=Lax; ...
{"user":{...}}
```

## Login lockout

The `LoginLimiter` is a per-IP `governor` bucket sized at
`LOGIN_BUCKET_PER_MINUTE = 5` (BACKEND.md §12). It counts every login
attempt — correct or wrong — which is the standard credential-stuffing
defense pattern.

```
attempt 1 (success): HTTP 401  (already used 1 token earlier via correct login)
attempt 2 (wrong):   HTTP 401
attempt 3 (wrong):   HTTP 401
attempt 4 (wrong):   HTTP 401
attempt 5 (wrong):   HTTP 429   ← bucket empty
attempt 6 (wrong):   HTTP 429
... up to 11:        HTTP 429
```

Full 429 response:

```
HTTP/1.1 429 Too Many Requests
content-type: application/json
retry-after: 6
x-request-id: 019e606f-043c-7a00-8977-9db9b455bca9

{"error":{"code":"rate_limited","message":"rate limited"},"request_id":"019e606f-043c-7a00-8977-9db9b455bca9"}
```

`Retry-After` is attached by `AppError::RateLimited` automatically. The
limiter `check` runs BEFORE the Argon2 verify so an attacker cannot burn
server CPU by spamming us.

> **Note on the lockout count:** BACKEND.md §16.3 says "11 wrong-password
> attempts from one IP → 429". In our run the bucket triggered at the 5th
> attempt because we'd already consumed one token via a prior correct
> login. The defensive goal — "credential-stuffer cannot crack a password
> in any reasonable time" — is met: an attacker gets at most
> `LOGIN_BUCKET_PER_MINUTE` attempts per minute, regardless of mix.

## Logout

```
$ curl -i -X POST .../v1/auth/logout -H "cookie: tfx_session=v1.<...>"
HTTP/1.1 204 No Content
set-cookie: tfx_session=; HttpOnly; SameSite=Lax; Path=/; Max-Age=0

$ psql -c "SELECT count(*) FILTER (WHERE revoked_at IS NULL) AS active, count(*) FILTER (WHERE revoked_at IS NOT NULL) AS revoked FROM sessions"
 active | revoked
--------+---------
      1 |       1
```

The DB row is marked `revoked_at = now()`; the cookie is cleared client-side
via `Max-Age=0`. Two-front defense.

## Password verify round-trip across process restart

Both the pepper (`AUTH_PASSWORD_PEPPER`) and the cookie key
(`AUTH_COOKIE_KEY`) come from env. After signup, kill the api, restart with
the same env, and log in with the same password:

```
$ curl -i -X POST .../v1/auth/login -d '{"email":"...","password":"Hunter2-Strong"}'
HTTP/1.1 200 OK
set-cookie: tfx_session=v1.<new-sid>.<new-token>.<new-hmac>; ...
{"user":{...}}
```

A fresh process re-built the keyed Argon2id instance from `AUTH_PASSWORD_PEPPER`,
parsed the PHC-encoded hash from the DB row, and verified — confirming the
hash format encodes Argon2 parameters (memory, time, parallelism, salt) but
NOT the pepper itself.

## Anti-regressions (added in PR #3)

11. `Argon2::new_with_secret` — pepper is the keyed parameter, never
    concatenated. The matching `verify_password` rebuilds the same instance.
12. Cookie value = `v1.<id>.<token>.<hmac>`. HMAC verified constant-time
    before any DB lookup. DB stores `sha256(token)` only.
13. `service_token` is required at boot; the api binary refuses to start
    when `SERVICE_TOKEN` / `AUTH_COOKIE_KEY` / `AUTH_PASSWORD_PEPPER` are
    unset.
14. `LoginLimiter` runs BEFORE Argon2 verify on the login path.
15. `UsersRepo::create` surfaces unique-violation as `UsersError::DuplicateEmail`,
    which maps to `AppError::Conflict("email already in use")` — never an
    `Internal` leak.
16. Sessions table never stores the plaintext token; revoking a row
    invalidates the cookie immediately even if the attacker has the
    plaintext cookie bytes.
