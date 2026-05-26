//! `AuditRepo` — append-only operations log.
//!
//! BACKEND.md §4 0020 + §22 rule 8: audit rows are written in the SAME
//! transaction as the operation they describe. PR #8 wires it in the
//! `checkout.session.completed` dispatcher.

use common::ids::{AuditId, UserId};
use sqlx::types::ipnetwork::IpNetwork;
use sqlx::PgPool;
use std::net::IpAddr;
use time::OffsetDateTime;

#[derive(Debug, Clone)]
pub struct AuditEntry {
    pub id: AuditId,
    pub actor_user_id: Option<UserId>,
    pub action: String,
    pub target_kind: String,
    pub target_id: String,
    pub metadata: serde_json::Value,
    pub created_at: OffsetDateTime,
}

#[derive(Clone)]
pub struct AuditRepo {
    pool: PgPool,
}

#[derive(Debug, thiserror::Error)]
pub enum AuditError {
    #[error("sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
}

impl AuditRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Record an action inside an existing tx. The hard-rule from
    /// BACKEND.md §22 item 8: never best-effort outside a tx.
    #[allow(clippy::too_many_arguments)]
    pub async fn record_in_tx<'c>(
        &self,
        tx: &mut sqlx::Transaction<'c, sqlx::Postgres>,
        actor_user_id: Option<UserId>,
        action: &str,
        target_kind: &str,
        target_id: &str,
        metadata: serde_json::Value,
        ip: Option<IpAddr>,
    ) -> Result<(), AuditError> {
        let id = uuid::Uuid::now_v7();
        let ip_net: Option<IpNetwork> = ip.map(Into::into);
        sqlx::query!(
            r#"
            INSERT INTO audit_log (id, actor_user_id, action, target_kind, target_id, metadata, ip)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            id,
            actor_user_id.map(|u| u.as_uuid()),
            action,
            target_kind,
            target_id,
            metadata,
            ip_net,
        )
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    pub async fn list_for_target(
        &self,
        target_kind: &str,
        target_id: &str,
    ) -> Result<Vec<AuditEntry>, AuditError> {
        let rows = sqlx::query!(
            r#"
            SELECT id, actor_user_id, action, target_kind, target_id, metadata, created_at
            FROM audit_log
            WHERE target_kind = $1 AND target_id = $2
            ORDER BY created_at DESC
            "#,
            target_kind,
            target_id
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| AuditEntry {
                id: AuditId::from_uuid(r.id),
                actor_user_id: r.actor_user_id.map(UserId::from_uuid),
                action: r.action,
                target_kind: r.target_kind,
                target_id: r.target_id,
                metadata: r.metadata,
                created_at: r.created_at,
            })
            .collect())
    }
}
