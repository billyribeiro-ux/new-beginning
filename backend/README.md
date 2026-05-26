# TradeFlex Backend

Rust + Axum + Tokio + sqlx + Postgres. See [`../BACKEND.md`](../BACKEND.md)
for the full plan and architectural rationale.

## Workspace layout

```
backend/
├── crates/
│   ├── common/          # Money, AppError, AlertSink, ids, config, request_id
│   ├── observability/   # tracing JSON + optional OTLP + Prometheus listener
│   └── http/            # Axum routes, middleware, AppState
└── bins/
    └── api/             # HTTP server binary
```

Future PRs add `storage`, `stripe-client`, `r2-client`, `mailer`, `jobs`,
`test-support`, and the `worker` + `seeder` binaries (BACKEND.md §21).

## Local development

```sh
# build
cargo build --workspace

# run all gates locally (must be green before commit)
just check

# run the api
just run
# → http://127.0.0.1:8080/healthz
# → http://127.0.0.1:9090/metrics  (separate listener)
```

Set `BIND_ADDR=127.0.0.1:8081` etc. to override.

## Config

Loaded from environment via `figment`. See `crates/common/src/config.rs`. PR
#1 needs no required vars — every field has a default appropriate for local
dev. Future PRs (`DATABASE_URL`, `STRIPE_SECRET_KEY`, etc.) become required.

| Var | Default | Status |
|---|---|---|
| `ENV` | `development` | active |
| `BIND_ADDR` | `0.0.0.0:8080` | active |
| `METRICS_BIND` | `0.0.0.0:9090` | active |
| `RUST_LOG` | `info,sqlx=warn,hyper=warn,h2=warn` | active |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | unset | active (optional) |
| `OTEL_SERVICE_NAME` | `tradeflex-api` | active |
| `DATABASE_URL` | unset | scaffolded (PR #2 makes it required) |
| `SERVICE_TOKEN` | unset | scaffolded (PR #3 makes it required) |

## Hard rules (CLAUDE.md, BACKEND.md §22)

1. Every `.rs` edit → rust-analyzer MCP diagnostics + `cargo fmt` +
   `cargo clippy --all-targets -- -D warnings`.
2. Money: `i64` cents everywhere. `mul_bps` / `mul_pct` / `mul_qty` return
   `Result` — never saturate.
3. `Internal` / `External` `AppError` never leaks the source string.
4. Migrations are forward-only.
5. PR cannot merge red CI.
