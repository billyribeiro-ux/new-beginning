# PR #4 — `/v1/me/*` (profile + sessions + change-email + change-password) — Launch Evidence

**Date:** 2026-05-25
**Scope:** BACKEND.md §21 row 4.

## Gates

| Gate | Result |
|---|---|
| `cargo fmt --all -- --check` | ✅ |
| `SQLX_OFFLINE=true cargo clippy --workspace --all-targets -- -D warnings` | ✅ |
| `cargo sqlx prepare --workspace --check` | ✅ |
| `cargo test --workspace` | ✅ **48/48** |
| `cargo deny check` | ✅ |
| signup → GET /v1/me round-trip | ✅ |
| PATCH /v1/me tri-state headline (set / clear-with-null / leave-when-absent) | ✅ |
| GET /v1/me/sessions lists active sessions with `current: true` flag | ✅ |
| POST /v1/me/sessions/revoke-others revokes peers, keeps self | ✅ |
| change-password rejects wrong current | ✅ |
| change-password revokes other sessions | ✅ |
| login with NEW password works after change | ✅ |
| login with OLD password rejected after change | ✅ |
| change-email stages an `email_verifications` row + logs `EMAIL_STUB` line | ✅ |

## Profile round-trip

```
$ curl -b cookies -H "x-service-token: ..." http://127.0.0.1:8081/v1/me
{"id":"019e607d-ffb3-...","email":"alex@example.com","name":"Alex Morgan",
 "headline":null,"timezone":"UTC","language":"en","role":"member",
 "email_verified_at":null,"created_at":"2026-05-25T18:55:30.74Z"}

$ curl -X PATCH -d '{"headline":"Day trader"}' .../v1/me
{"...","headline":"Day trader",...}

$ curl -X PATCH -d '{"headline":null}' .../v1/me
{"...","headline":null,...}

$ curl -X PATCH -d '{"name":"Alex M."}' .../v1/me
{"...","name":"Alex M.","headline":null,...}   # absent ⇒ left alone
```

The tri-state behavior on `headline` requires a `double_option` serde helper:
default serde collapses absent and `null` to `None`. See
`crates/http/src/routes/me/profile.rs::double_option`.

## Sessions list + revoke-others

```
$ curl -b jar_A .../v1/me/sessions
{
  "sessions": [
    {"id":"019e607e-3747-...","current":true,  "ip":"127.0.0.1", ...},
    {"id":"019e607e-3f90-...","current":false, "ip":"127.0.0.1", ...},
    {"id":"019e607d-ffba-...","current":false, "ip":null,        ...}
  ]
}

$ curl -X POST -b jar_A .../v1/me/sessions/revoke-others
{"revoked":2}

$ curl -b jar_B .../v1/me
HTTP 401      # peer session revoked

$ curl -b jar_A .../v1/me
HTTP 200      # caller's session intact
```

## change-password (rejects wrong, rotates others on success)

```
$ curl -X POST -d '{"current_password":"wrong","new_password":"Brand-New-99"}' .../v1/me/change-password
HTTP 401

$ curl -X POST -d '{"current_password":"Hunter2-Strong","new_password":"Brand-New-Pass-99"}' .../v1/me/change-password
{"other_sessions_revoked":1}

$ curl -b jar_A .../v1/me          # caller still in
HTTP 200
$ curl -b jar_C .../v1/me          # peer logged out
HTTP 401

$ curl -X POST -d '{"email":"...","password":"Brand-New-Pass-99"}' .../v1/auth/login   HTTP 200
$ curl -X POST -d '{"email":"...","password":"Hunter2-Strong"}'    .../v1/auth/login   HTTP 401
```

Both verify-current AND hash-new run under the `hash_semaphore` permit; the
limiter on `/v1/auth/login` runs BEFORE Argon2 so an attacker who can't see
`/v1/me/change-password` (no session) cannot burn CPU there.

## change-email (staged; EMAIL_STUB line)

```
$ curl -X POST -d '{"new_email":"alex+new@example.com"}' .../v1/me/change-email
HTTP/1.1 202 Accepted
{"status":"verification_required","expires_at":"2026-05-25T19:55:30.804789Z"}

$ psql -c "SELECT count(*) FROM email_verifications WHERE kind='email_change'"
 1

$ grep EMAIL_STUB tfx-api-pr4.log
{"level":"WARN","fields":{"message":"EMAIL_STUB email_change — replace with mailer enqueue in PR-X",
 "verification_id":"019e607d-fff4-...","user_id":"019e607d-ffb3-...",
 "new_email":"alex+new@example.com","kind":"email_change",
 "token":"77j-dtegIAupFQmlqekLrgdEmWqyEzv5Z3wnc9b0rMc"}}
```

The plaintext token is grep-able in dev. When the mailer crate lands, the
exact same row is consumed by `POST /v1/auth/verify-email`; the only change
will be replacing the `tracing::warn!` call with a `mailer::enqueue`.

## Anti-regressions (added in PR #4)

17. PATCH JSON bodies that need a tri-state field MUST use the
    `double_option` deserializer; serde collapses absent and `null` to
    `None` by default.
18. Password change requires verify-current AND revokes all-but-current.
19. `/v1/me/*` endpoints all run behind `require_auth`. A missing/invalid
    cookie → 401; never silently degraded.
20. `change-email` is staged via `email_verifications`; the row's
    `token_hash` is `sha256(plaintext)`, plaintext never persisted.
