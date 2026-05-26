# PR #5 — TOTP enable/confirm/disable + backup codes — Launch Evidence

**Date:** 2026-05-25
**Scope:** BACKEND.md §21 row 5.

## Gates

| Gate | Result |
|---|---|
| `cargo fmt --all -- --check` | ✅ |
| `SQLX_OFFLINE=true cargo clippy --workspace --all-targets -- -D warnings` | ✅ |
| `cargo sqlx prepare --workspace --check` | ✅ |
| `cargo test --workspace` | ✅ **70/70** |
| `cargo deny check` | ✅ |
| Every BACKEND.md §16.3 2FA bullet | ✅ (see below) |

## Live 2FA flow (BACKEND.md §16.3 checklist)

### 1. Enable → scan QR → confirm → 10 backup codes shown ONCE

```
$ curl -X POST .../v1/me/2fa/enable
{
  "otpauth_uri": "otpauth://totp/TradeFlex:alex%40example.com?secret=NGFV56NKUIKBBVQNMSXMBDYCCDDYFV5D&issuer=TradeFlex",
  "qr_data_url": "data:image/png;base64,...4614 chars..."
}

# Compute current code from the otpauth secret (e.g. python /tmp/totp_cli.py)
$ curl -X POST .../v1/me/2fa/confirm -d '{"code":"365040"}'
{"backup_codes":[
  "0VWX79K1RB","7WJWY8M0D9","JCCGDZH51M","JVDDVPQR2Y","EHH6C057FY",
  "K9HCQGDZC7","NAPK03JRGS","KQSRB816GZ","PDGF9FE5HP","H4FA8ZW77R"
]}
```

The codes use the Crockford base32 alphabet (no `I`/`L`/`O`/`U` ambiguity).
**The plaintext is returned exactly once** — the DB stores only Argon2id
hashes (keyed-pepper).

### 2. `users.totp_enabled_at IS NOT NULL` + 10 hashed slots

```
$ psql -c "SELECT totp_enabled_at IS NOT NULL, array_length(twofa_backup_codes_hash, 1) FROM users"
 twofa_on | slots
----------+-------
 t        | 10
```

### 3. Two-step login: stage 1 returns `{step:"totp",tx_id}`, NO cookie

```
$ curl -X POST .../v1/auth/login -d '{"email":"...","password":"..."}'
{"step":"totp","tx_id":"v1.AZ5gkMix...3.AAAAAGoUoXs.h6dai_kc...Bc"}
```

No `Set-Cookie` header. The caller must POST to `/v1/auth/login/totp`.

### 4. Skew=1 accepts the **previous** 30-s step

```
$ python3 totp_cli.py SECRET -30   # generate code valid 30 s ago
970129
$ curl -X POST .../v1/auth/login/totp -d '{"tx_id":"v1...","code":"970129"}'
HTTP/1.1 200 OK
set-cookie: tfx_session=v1...; HttpOnly; SameSite=Lax; ...
```

### 5. Skew=2 rejects a code from **two** steps ago

```
$ python3 totp_cli.py SECRET -60   # code valid 60 s ago
139030
$ curl -o /dev/null -w "%{http_code}\n" -X POST .../v1/auth/login/totp -d '{"tx_id":"v1...","code":"139030"}'
401
```

### 6. Backup code consumes a slot; replay → 401

```
$ curl -X POST .../v1/auth/login/totp -d '{"tx_id":"v1...","code":"H4FA8ZW77R"}'
HTTP/1.1 200 OK

$ psql -c "SELECT array_length(twofa_backup_codes_hash, 1) total, \
   (SELECT count(*) FROM unnest(twofa_backup_codes_hash) h WHERE octet_length(h)=1) consumed FROM users"
 total | consumed
-------+----------
    10 |        1

# Replay the same code:
$ curl -o /dev/null -w "%{http_code}\n" -X POST .../v1/auth/login/totp -d '{"tx_id":"v1...","code":"H4FA8ZW77R"}'
401
```

The slot is replaced with all-zero `BYTEA` (length 1) — the array length
stays at 10 so the UI can render "9/10 remaining". The CAS in
`UsersRepo::consume_backup_code_slot` makes two concurrent uses race so at
most one wins.

### 7. Disable requires BOTH current password AND current code

```
# Wrong password, right code         → 401
# Right password, wrong code         → 401
# Right password, right code         → 204

$ psql -c "SELECT totp_enabled_at IS NULL twofa_off, totp_secret_encrypted IS NULL secret_cleared, twofa_backup_codes_hash IS NULL codes_cleared FROM users"
 twofa_off | secret_cleared | codes_cleared
-----------+----------------+---------------
 t         | t              | t
```

### 8. Post-disable login is single-step again

```
$ curl -X POST .../v1/auth/login -d '{"email":"...","password":"..."}'
{"step":"done","user":{...}}
set-cookie: tfx_session=...
```

The `tag = "step"` adjacently-tagged enum gives the BFF a stable shape to
branch on without sniffing fields.

## Anti-regressions (added in PR #5)

21. TOTP params are pinned literals in the constructor (SHA-1, 6 digits,
    skew=1, 30s period). Never rely on a crate default.
22. TOTP secrets are encrypted at rest with XChaCha20-Poly1305. The DB
    blob is `[nonce:24][ciphertext+tag:36]`; tamper or wrong-key produces
    a `Decrypt` error (never silent garbage).
23. Backup codes use Crockford base32 (no `I`/`L`/`O`/`U`) and are
    Argon2id-hashed with the same keyed-pepper instance as passwords.
24. Backup-code consumption is a CAS on the pre-image bytes —
    concurrent uses cannot double-redeem.
25. Disable requires current-password AND current-code (TOTP OR backup).
26. Pending-TOTP token (`tx_id`) is HMAC-signed with a domain separator
    `tfx_pending_totp.` so it cannot be confused with a session cookie
    minted from the same `AUTH_COOKIE_KEY`.
27. `AUTH_TOTP_KEY` is required at api boot — missing → fail-fast.
28. `LoginLimiter` is consulted at BOTH stages (1 and 2) of two-step
    login so an attacker who got past password cannot pound TOTP codes.
