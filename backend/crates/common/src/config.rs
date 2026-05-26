//! Typed config loader.
//!
//! BACKEND.md §13: every env var the binary expects, grouped. Loaded once at
//! startup via `figment` (env + optional `.env` in dev). Secrets are wrapped
//! in `secrecy::SecretString` and never logged.
//!
//! PR #1 ships only the fields the api binary needs *today* — bind address,
//! environment, observability vars, the optional service token, and a
//! database url placeholder. Subsequent PRs append fields here as they need
//! them; no field is added speculatively.

use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use ipnet::IpNet;
use secrecy::SecretString;
use serde::{Deserialize, Deserializer};
use std::net::SocketAddr;
use std::str::FromStr;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    /// Logical environment name. Free-form, used in logs and metric labels.
    /// Common values: `"development"`, `"staging"`, `"production"`.
    #[serde(default = "default_env")]
    pub env: String,

    /// `BIND_ADDR=0.0.0.0:8080`. The api binary listens here.
    #[serde(default = "default_bind_addr")]
    pub bind_addr: SocketAddr,

    /// Prometheus metrics listener. Separate port from `bind_addr` so a public
    /// LB cannot accidentally expose internal metrics.
    #[serde(default = "default_metrics_bind")]
    pub metrics_bind: SocketAddr,

    /// `RUST_LOG=info,sqlx=warn`. Read by `EnvFilter`. Optional; the
    /// observability crate falls back to a default if unset.
    #[serde(default)]
    pub rust_log: Option<String>,

    /// `OTEL_EXPORTER_OTLP_ENDPOINT`. If unset, the OTLP layer is not
    /// installed (BACKEND.md §15) — stdout JSON tracing still runs.
    #[serde(default)]
    pub otel_exporter_otlp_endpoint: Option<String>,

    /// `OTEL_SERVICE_NAME`. Defaults per binary in observability init.
    #[serde(default)]
    pub otel_service_name: Option<String>,

    /// `DATABASE_URL`. Optional at the config layer; the api/seeder binaries
    /// require it at boot (`storage::build_pool` returns `PoolError::MissingUrl`).
    /// Keeping it Optional here lets tests construct a Config without a DB.
    #[serde(default)]
    pub database_url: Option<SecretString>,

    /// `DATABASE_MAX_CONN` — pool ceiling. BACKEND.md §13 default = 20.
    #[serde(default = "default_database_max_conn")]
    pub database_max_conn: u32,

    /// `DATABASE_MIN_CONN` — pool floor. BACKEND.md §13 default = 2.
    #[serde(default = "default_database_min_conn")]
    pub database_min_conn: u32,

    /// `SERVICE_TOKEN`. The BFF-to-api shared secret (BACKEND.md §1.1).
    /// Optional at the config layer; the api binary's `main` aborts boot if
    /// missing (PR #3 makes the middleware fail-closed).
    #[serde(default)]
    pub service_token: Option<SecretString>,

    /// `SERVICE_TOKEN_IP_ALLOWLIST=10.0.0.0/8,192.168.0.0/16`. CIDR list. An
    /// empty list disables the IP check (local dev).
    #[serde(default, deserialize_with = "deserialize_cidr_list")]
    pub service_token_ip_allowlist: Vec<IpNet>,

    /// `AUTH_COOKIE_KEY`. HMAC key for the session-cookie signature
    /// (BACKEND.md §7.1). Required at api boot.
    #[serde(default)]
    pub auth_cookie_key: Option<SecretString>,

    /// `AUTH_PASSWORD_PEPPER`. Argon2id `secret` parameter
    /// (BACKEND.md §1.2). Required at api boot.
    #[serde(default)]
    pub auth_password_pepper: Option<SecretString>,

    /// `AUTH_TOTP_KEY`. XChaCha20-Poly1305 key for TOTP secrets at rest.
    /// Required at api boot (PR #5).
    #[serde(default)]
    pub auth_totp_key: Option<SecretString>,

    /// `STRIPE_SECRET_KEY`. Required at api boot (PR #7).
    #[serde(default)]
    pub stripe_secret_key: Option<SecretString>,

    /// `STRIPE_WEBHOOK_SECRET`. Required at api boot (PR #7).
    #[serde(default)]
    pub stripe_webhook_secret: Option<SecretString>,

    /// `STRIPE_API_VERSION`. Pinned in `stripe_client::api::DEFAULT_API_VERSION`
    /// when unset.
    #[serde(default)]
    pub stripe_api_version: Option<String>,

    /// Base URL for `success_url` / `cancel_url` on Checkout Sessions.
    /// Defaults to `http://127.0.0.1:5173` (Vite dev server) so local dev
    /// works without setting it.
    #[serde(default = "default_public_site_url")]
    pub public_site_url: String,

    // R2 / S3 object store (BACKEND.md §9 / §13). All four must be set to
    // enable the real client; otherwise the api binary falls back to a
    // RecordingObjectStore + logs a WARN. The fallback exists so local
    // dev without R2 creds still boots, NOT as a prod-safe default.
    #[serde(default)]
    pub r2_endpoint: Option<String>,
    #[serde(default)]
    pub r2_bucket: Option<String>,
    #[serde(default)]
    pub r2_access_key_id: Option<SecretString>,
    #[serde(default)]
    pub r2_secret_access_key: Option<SecretString>,

    /// `AUTH_SESSION_TTL_DAYS`. Cookie + DB row TTL.
    #[serde(default = "default_auth_session_ttl_days")]
    pub auth_session_ttl_days: u32,

    /// `AUTH_COOKIE_DOMAIN`. Optional domain attribute on the cookie. Unset
    /// means the cookie is host-scoped (correct for local dev and
    /// preview deploys).
    #[serde(default)]
    pub auth_cookie_domain: Option<String>,
}

fn default_env() -> String {
    "development".into()
}

fn default_bind_addr() -> SocketAddr {
    "0.0.0.0:8080".parse().expect("default bind addr is valid")
}

fn default_metrics_bind() -> SocketAddr {
    "0.0.0.0:9090"
        .parse()
        .expect("default metrics bind is valid")
}

fn default_database_max_conn() -> u32 {
    20
}

fn default_database_min_conn() -> u32 {
    2
}

fn default_auth_session_ttl_days() -> u32 {
    30
}

fn default_public_site_url() -> String {
    "http://127.0.0.1:5173".into()
}

/// Deserialize `"10.0.0.0/8,192.168.0.0/16"` → `Vec<IpNet>`. Empty string
/// produces an empty vec. Each entry is parsed by `IpNet::from_str`.
fn deserialize_cidr_list<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<IpNet>, D::Error> {
    let s: Option<String> = Option::deserialize(d)?;
    let Some(s) = s else { return Ok(Vec::new()) };
    if s.trim().is_empty() {
        return Ok(Vec::new());
    }
    s.split(',')
        .map(|part| {
            IpNet::from_str(part.trim())
                .map_err(|e| serde::de::Error::custom(format!("invalid CIDR {part:?}: {e}")))
        })
        .collect()
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// Boxed because `figment::Error` carries a sizeable backtrace and
    /// clippy's `result_large_err` lint flags it otherwise. There is exactly
    /// one config-load call per process; the allocation is trivial.
    #[error("config: {0}")]
    Figment(#[from] Box<figment::Error>),
}

impl From<figment::Error> for ConfigError {
    fn from(e: figment::Error) -> Self {
        ConfigError::Figment(Box::new(e))
    }
}

impl Config {
    /// Load from environment, with optional `backend/config.toml` overlay for
    /// local dev. Failing to parse aborts boot — there is no fallback.
    pub fn from_env() -> Result<Self, ConfigError> {
        let cfg: Config = Figment::new()
            .merge(Toml::file("config.toml"))
            .merge(Env::raw())
            .extract()?;
        Ok(cfg)
    }

    pub fn is_production(&self) -> bool {
        self.env == "production"
    }
}

#[cfg(test)]
// `Jail::expect_with` requires a closure returning `Result<(), figment::Error>`.
// We don't own that signature, so boxing isn't applicable — allow the lint
// for this test module only.
#[allow(clippy::result_large_err)]
mod tests {
    use super::*;
    use figment::Jail;

    #[test]
    fn defaults_when_unset() {
        Jail::expect_with(|jail| {
            // Clear potentially inherited vars.
            jail.clear_env();
            let cfg: Config = Figment::new().merge(Env::raw()).extract().unwrap();
            assert_eq!(cfg.env, "development");
            assert_eq!(cfg.bind_addr.port(), 8080);
            assert_eq!(cfg.metrics_bind.port(), 9090);
            assert!(cfg.database_url.is_none());
            assert!(cfg.service_token.is_none());
            assert_eq!(cfg.database_max_conn, 20);
            assert_eq!(cfg.database_min_conn, 2);
            Ok(())
        });
    }

    #[test]
    fn env_overrides_defaults() {
        Jail::expect_with(|jail| {
            jail.clear_env();
            jail.set_env("ENV", "production");
            jail.set_env("BIND_ADDR", "127.0.0.1:9999");
            let cfg: Config = Figment::new().merge(Env::raw()).extract().unwrap();
            assert!(cfg.is_production());
            assert_eq!(cfg.bind_addr.port(), 9999);
            Ok(())
        });
    }

    #[test]
    fn secret_url_loaded_but_not_in_debug() {
        Jail::expect_with(|jail| {
            jail.clear_env();
            jail.set_env("DATABASE_URL", "postgres://u:hunter2@localhost/db");
            let cfg: Config = Figment::new().merge(Env::raw()).extract().unwrap();
            // SecretString's Debug must redact.
            let dbg = format!("{:?}", cfg.database_url);
            assert!(!dbg.contains("hunter2"), "secret leaked: {dbg}");
            Ok(())
        });
    }
}
