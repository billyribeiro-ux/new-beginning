//! `SessionsRepo` — opaque-token session lookup.
//!
//! BACKEND.md §7.1 / §7.5. The DB stores `sha256(token)` only; the plaintext
//! lives in the cookie. A DB dump cannot resurrect a session.
//!
//! `last_seen_at` is updated inline on every load in PR #3. BACKEND.md §7.1
//! calls for 60 s coalescing via `moka`; that optimization lands in a
//! follow-up once we have a workload to measure against.

use common::ids::{SessionId, UserId};
use sqlx::PgPool;
use std::net::IpAddr;
use time::{Duration, OffsetDateTime};

#[derive(Debug, Clone)]
pub struct ActiveSession {
    pub id: SessionId,
    pub user_id: UserId,
    pub expires_at: OffsetDateTime,
}

#[derive(Debug, Clone)]
pub struct SessionListItem {
    pub id: SessionId,
    pub user_agent: Option<String>,
    pub ip: Option<IpAddr>,
    pub last_seen_at: OffsetDateTime,
    pub expires_at: OffsetDateTime,
}

#[derive(Clone)]
pub struct SessionsRepo {
    pool: PgPool,
    default_ttl_days: i64,
}

#[derive(Debug, thiserror::Error)]
pub enum SessionsError {
    #[error("sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
}

impl SessionsRepo {
    pub fn new(pool: PgPool, default_ttl_days: i64) -> Self {
        Self {
            pool,
            default_ttl_days,
        }
    }

    /// Insert a new session row. The plaintext token is NEVER passed in;
    /// the caller hashes it (via `common::auth::session_cookie::issue`) and
    /// forwards the hash.
    pub async fn create(
        &self,
        id: SessionId,
        user_id: UserId,
        token_hash: &[u8; 32],
        user_agent: Option<&str>,
        ip: Option<IpAddr>,
    ) -> Result<ActiveSession, SessionsError> {
        let expires_at = OffsetDateTime::now_utc() + Duration::days(self.default_ttl_days);
        let ip_net: Option<sqlx::types::ipnetwork::IpNetwork> = ip.map(Into::into);
        let row = sqlx::query!(
            r#"
            INSERT INTO sessions (id, user_id, token_hash, user_agent, ip, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, user_id, expires_at
            "#,
            id.as_uuid(),
            user_id.as_uuid(),
            &token_hash[..],
            user_agent,
            ip_net,
            expires_at,
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(ActiveSession {
            id: SessionId::from_uuid(row.id),
            user_id: UserId::from_uuid(row.user_id),
            expires_at: row.expires_at,
        })
    }

    /// Look up a session by its token hash. Returns `None` if the session is
    /// missing, revoked, or expired. Inline-updates `last_seen_at` on hit so
    /// the sessions list reflects activity.
    pub async fn load_by_token_hash(
        &self,
        token_hash: &[u8; 32],
    ) -> Result<Option<ActiveSession>, SessionsError> {
        let row = sqlx::query!(
            r#"
            UPDATE sessions
            SET last_seen_at = now()
            WHERE token_hash = $1
              AND revoked_at IS NULL
              AND expires_at > now()
            RETURNING id, user_id, expires_at
            "#,
            &token_hash[..]
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| ActiveSession {
            id: SessionId::from_uuid(r.id),
            user_id: UserId::from_uuid(r.user_id),
            expires_at: r.expires_at,
        }))
    }

    /// Revoke a single session (logout / dashboard "sign out this device").
    /// Returns `Ok(true)` if the row was updated, `Ok(false)` if it was
    /// already revoked or missing — both safe to surface as 204 to the user.
    pub async fn revoke(&self, id: SessionId) -> Result<bool, SessionsError> {
        let res = sqlx::query!(
            r#"
            UPDATE sessions SET revoked_at = now()
            WHERE id = $1 AND revoked_at IS NULL
            "#,
            id.as_uuid()
        )
        .execute(&self.pool)
        .await?;
        Ok(res.rows_affected() > 0)
    }

    /// "Sign out every other device" — revokes every active session for a
    /// user except the one currently in use. BACKEND.md §7.5.
    pub async fn revoke_all_except(
        &self,
        user_id: UserId,
        keep: SessionId,
    ) -> Result<u64, SessionsError> {
        let res = sqlx::query!(
            r#"
            UPDATE sessions
            SET revoked_at = now()
            WHERE user_id = $1
              AND id <> $2
              AND revoked_at IS NULL
            "#,
            user_id.as_uuid(),
            keep.as_uuid(),
        )
        .execute(&self.pool)
        .await?;
        Ok(res.rows_affected())
    }

    pub async fn list_active(
        &self,
        user_id: UserId,
    ) -> Result<Vec<SessionListItem>, SessionsError> {
        let rows = sqlx::query!(
            r#"
            SELECT id, user_agent, ip, last_seen_at, expires_at
            FROM sessions
            WHERE user_id = $1 AND revoked_at IS NULL AND expires_at > now()
            ORDER BY last_seen_at DESC
            "#,
            user_id.as_uuid()
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| SessionListItem {
                id: SessionId::from_uuid(r.id),
                user_agent: r.user_agent,
                ip: r.ip.map(|n| n.ip()),
                last_seen_at: r.last_seen_at,
                expires_at: r.expires_at,
            })
            .collect())
    }
}
